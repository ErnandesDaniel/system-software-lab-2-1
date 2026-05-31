<?php

/**
 * PHP pipe-клиент для MyLang сервера (lab-4)
 *
 * Компиляция (NASM):
 *   cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t nasm
 *
 * Компиляция (JVM):
 *   cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t jvm
 *
 * Запуск:
 *   php labs-examples/vitrual-machines/lab-4/input.php [--protocol text|binary] [--target nasm|jvm]
 *
 * Протоколы:
 *   text   — "opcode key value\n" -> "OK [payload]\n" (по умолчанию)
 *   binary — бинарный протокол [magic:1B][opcode:1B][klen:2B LE][key][vlen:2B LE][val]
 *            -> [status:1B][plen:2B LE][payload]
 *
 * Цели:
 *   nasm   — запустить скомпилированный .exe (по умолчанию)
 *   jvm    — запустить через java -cp <output> RuntimeStub
 */

const BINARY_MAGIC = "\xBF";

function getArgs(): array {
    $argv = $_SERVER['argv'] ?? [];
    $protocol = 'text';
    $target = 'nasm';
    $i = 1;
    while ($i < count($argv)) {
        switch ($argv[$i]) {
            case '--protocol':
                if ($i + 1 < count($argv)) {
                    $protocol = $argv[$i + 1];
                    $i += 2;
                } else {
                    $i++;
                }
                break;
            case '--target':
                if ($i + 1 < count($argv)) {
                    $target = $argv[$i + 1];
                    $i += 2;
                } else {
                    $i++;
                }
                break;
            case '--help':
                printHelp();
                exit(0);
            default:
                $i++;
        }
    }
    if (!in_array($protocol, ['text', 'binary'], true)) {
        echo "Unknown protocol: $protocol (use text or binary)\n";
        exit(1);
    }
    if (!in_array($target, ['nasm', 'jvm'], true)) {
        echo "Unknown target: $target (use nasm or jvm)\n";
        exit(1);
    }
    return [$protocol, $target];
}

function startDaemon(string $protocol, string $target): array {
    if ($target === 'jvm') {
        $cmd = "java -cp \"output\" RuntimeStub";
    } else {
        $cmd = PHP_OS_FAMILY === 'Windows'
            ? '.\\output\\program.exe'
            : './output/program';
    }

    $descriptors = [
        0 => ['pipe', 'r'],  // stdin
        1 => ['pipe', 'w'],  // stdout
        2 => ['pipe', 'w'],  // stderr
    ];

    $process = proc_open($cmd, $descriptors, $pipes, getcwd());
    if (!is_resource($process)) {
        throw new RuntimeException("Failed to start: $cmd");
    }

    // For binary protocol, send magic byte to tell server to use binary mode
    if ($protocol === 'binary') {
        fwrite($pipes[0], BINARY_MAGIC);
        fflush($pipes[0]);
    }

    return [$process, $pipes];
}

// ─── Text protocol ────────────────────────────────────────────────────────────

function sendText($stdin, $stdout, string $opcode, string $key = '', string $value = ''): ?string {
    $line = $opcode;
    if ($key !== '') { $line .= " $key"; }
    if ($value !== '') { $line .= " $value"; }
    $line .= "\n";
    fwrite($stdin, $line);
    fflush($stdin);

    $response = rtrim(fgets($stdout));
    if ($response === false || $response === '') {
        throw new RuntimeException("No response");
    }

    if (str_starts_with($response, 'ERR ')) {
        throw new RuntimeException(substr($response, 4));
    }
    if (str_starts_with($response, 'OK')) {
        $payload = trim(substr($response, 2));
        return $payload !== '' ? $payload : null;
    }
    throw new RuntimeException("Bad response: $response");
}

// ─── Binary protocol ──────────────────────────────────────────────────────────

function sendBinary($stdin, $stdout, string $opcode, string $key = '', string $value = ''): ?string {
    $opcodeMap = ['create' => 1, 'get' => 2, 'set' => 3, 'delete' => 4, 'list' => 5, 'exit' => 6];
    $op = $opcodeMap[$opcode] ?? 0;

    // Pack: opcode(1B) + key_len(2B LE) + key + val_len(2B LE) + val
    $packet = pack('C', $op)
            . pack('v', strlen($key))
            . $key
            . pack('v', strlen($value))
            . $value;
    fwrite($stdin, $packet);
    fflush($stdin);

    // Read response: status(1B) + payload_len(2B LE) + payload
    $header = fread($stdout, 3);
    if ($header === false || strlen($header) < 3) {
        throw new RuntimeException("No response");
    }
    $status = unpack('C', $header[0])[1];
    $payloadLen = unpack('v', $header[1] . $header[2])[1];
    $payload = '';
    if ($payloadLen > 0) {
        $payload = fread($stdout, $payloadLen);
    }

    if ($status === 1) {
        throw new RuntimeException($payload);
    }
    return $payload !== '' ? $payload : null;
}

// ─── Help & Main ──────────────────────────────────────────────────────────────

function printHelp(): void {
    echo "
PHP pipe -> MyLang Daemon
=========================

Usage:
  php input.php [--protocol text|binary] [--target nasm|jvm]

Flags:
  --protocol text|binary   Protocol mode (default: text)
  --target nasm|jvm        Compilation target (default: nasm)

Commands:
  create <key> <value>      Create entry
  get <key>                 Get value
  set <key> <value>         Update entry
  delete <key>              Delete key
  list                      List all keys
  exit                      Exit
  help                      This help
";
}

function main(): void {
    [$protocol, $target] = getArgs();

    echo "=== PHP pipe -> MyLang Daemon ($protocol, target=$target) ===\n\n";

    try {
        [$process, $pipes] = startDaemon($protocol, $target);
    } catch (Exception $e) {
        echo "[ERR] " . $e->getMessage() . "\n";
        exit(1);
    }

    [$stdin, $stdout, $stderr] = $pipes;
    echo "[OK] Daemon started\n\n";

    $send = $protocol === 'binary' ? 'sendBinary' : 'sendText';

    while (true) {
        echo "> ";
        $input = fgets(STDIN);
        if ($input === false) break;
        $line = trim($input);
        if ($line === '') continue;

        $parts = explode(' ', $line);
        $cmd = array_shift($parts);

        try {
            switch ($cmd) {
                case 'exit':
                case 'quit':
                    fwrite($stdin, $protocol === 'binary' ? pack('C', 6) : "exit\n");
                    fflush($stdin);
                    fclose($stdin);
                    fclose($stdout);
                    fclose($stderr);
                    proc_close($process);
                    echo "Bye!\n";
                    exit(0);

                case 'help':
                    printHelp();
                    break;

                case 'create':
                    $send($stdin, $stdout, 'create', array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;

                case 'get':
                    $r = $send($stdin, $stdout, 'get', array_shift($parts) ?? '');
                    echo "  value: " . ($r ?? '(empty)') . "\n";
                    break;

                case 'set':
                    $send($stdin, $stdout, 'set', array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;

                case 'delete':
                    $send($stdin, $stdout, 'delete', array_shift($parts) ?? '');
                    echo "  ok\n";
                    break;

                case 'list':
                    $r = $send($stdin, $stdout, 'list');
                    if ($r === null || $r === '') {
                        echo "  (no keys)\n";
                    } else {
                        echo "  keys:\n";
                        foreach (explode(',', $r) as $e) {
                            echo "    - $e\n";
                        }
                    }
                    break;

                default:
                    echo "Unknown: $cmd (type 'help')\n";
            }
        } catch (Exception $e) {
            echo "  ERROR: " . $e->getMessage() . "\n";
        }
    }

    fclose($stdin);
    fclose($stdout);
    fclose($stderr);
    proc_close($process);
}

main();

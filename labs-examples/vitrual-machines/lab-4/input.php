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
 *   php labs-examples/vitrual-machines/lab-4/input.php [--target nasm|jvm]
 *
 * Протокол: "opcode key value\n" -> "OK [payload]\n" / "ERR msg\n"
 *
 * Цели:
 *   nasm   — запустить скомпилированный .exe (по умолчанию)
 *   jvm    — запустить через java -cp <output> RuntimeStub
 */

function getArgs(): string {
    $argv = $_SERVER['argv'] ?? [];
    $target = 'nasm';
    $i = 1;
    while ($i < count($argv)) {
        switch ($argv[$i]) {
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
    if (!in_array($target, ['nasm', 'jvm'], true)) {
        echo "Unknown target: $target (use nasm or jvm)\n";
        exit(1);
    }
    return $target;
}

function startDaemon(string $target): array {
    if ($target === 'jvm') {
        $cmd = "java -cp \"output\" RuntimeStub";
    } else {
        $cmd = PHP_OS_FAMILY === 'Windows'
            ? '.\\output\\program.exe'
            : './output/program';
    }

    $descriptors = [
        0 => ['pipe', 'r'],
        1 => ['pipe', 'w'],
        2 => ['pipe', 'w'],
    ];

    $process = proc_open($cmd, $descriptors, $pipes, getcwd());
    if (!is_resource($process)) {
        throw new RuntimeException("Failed to start: $cmd");
    }

    return [$process, $pipes];
}

function sendCmd($stdin, $stdout, string $opcode, string $key = '', string $value = ''): ?string {
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

function printHelp(): void {
    echo "
PHP pipe -> MyLang Daemon
=========================

Usage:
  php input.php [--target nasm|jvm]

Flags:
  --target nasm|jvm   Compilation target (default: nasm)

Commands:
  create <key> <value>   Create entry
  get <key>              Get value
  set <key> <value>      Update entry
  delete <key>           Delete key
  list                   List all keys
  exit                   Exit
  help                   This help
";
}

function main(): void {
    $target = getArgs();

    echo "=== PHP pipe -> MyLang Daemon (text, target=$target) ===\n\n";

    try {
        [$process, $pipes] = startDaemon($target);
    } catch (Exception $e) {
        echo "[ERR] " . $e->getMessage() . "\n";
        exit(1);
    }

    [$stdin, $stdout, $stderr] = $pipes;
    echo "[OK] Daemon started\n\n";

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
                    fwrite($stdin, "exit\n");
                    fflush($stdin);
                    echo "Bye!\n";
                    exit(0);

                case 'help':
                    printHelp();
                    break;

                case 'create':
                    sendCmd($stdin, $stdout, 'create', array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;

                case 'get':
                    $r = sendCmd($stdin, $stdout, 'get', array_shift($parts) ?? '');
                    echo "  value: " . ($r ?? '(empty)') . "\n";
                    break;

                case 'set':
                    sendCmd($stdin, $stdout, 'set', array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;

                case 'delete':
                    sendCmd($stdin, $stdout, 'delete', array_shift($parts) ?? '');
                    echo "  ok\n";
                    break;

                case 'list':
                    $r = sendCmd($stdin, $stdout, 'list');
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

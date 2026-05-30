<?php

/**
 * PHP pipe-клиент для MyLang сервера (lab-4)
 *
 * Компиляция:
 *   cargo run -- labs-examples/vitrual-machines/lab-4/input.mylang -o output -t nasm
 *
 * Запуск:
 *   php labs-examples/vitrual-machines/lab-4/input.php
 *
 * Протокол: "opcode key value\n" -> "OK [payload]\n"
 */

function startDaemon(): array {
    $cmd = PHP_OS_FAMILY === 'Windows'
        ? '.\\output\\program.exe'
        : './output/program';

    $descriptors = [
        0 => ['pipe', 'r'],  // stdin
        1 => ['pipe', 'w'],  // stdout
        2 => ['pipe', 'w'],  // stderr
    ];

    $process = proc_open($cmd, $descriptors, $pipes, getcwd());
    if (!is_resource($process)) {
        throw new RuntimeException("Failed to start: $cmd");
    }

    return [$process, $pipes];
}

function send(string $stdin, $stdout, string $opcode, string $key = '', string $value = ''): ?string {
    fwrite($stdin, "$opcode $key $value\n");
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
    if (in_array('--help', $_SERVER['argv'] ?? [])) {
        printHelp();
        return;
    }

    echo "=== PHP pipe -> MyLang Daemon ===\n\n";

    try {
        [$process, $pipes] = startDaemon();
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
                    send($stdin, $stdout, 'exit');
                    echo "Bye!\n";
                    break 2;

                case 'help':
                    printHelp();
                    break;

                case 'create':
                    send($stdin, $stdout, 'create', array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;

                case 'get':
                    $r = send($stdin, $stdout, 'get', array_shift($parts) ?? '');
                    echo "  value: " . ($r ?? '(empty)') . "\n";
                    break;

                case 'set':
                    send($stdin, $stdout, 'set', array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;

                case 'delete':
                    send($stdin, $stdout, 'delete', array_shift($parts) ?? '');
                    echo "  ok\n";
                    break;

                case 'list':
                    $r = send($stdin, $stdout, 'list');
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

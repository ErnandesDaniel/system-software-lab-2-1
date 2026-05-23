<?php

require_once __DIR__ . '/shm_client.php';

function printHelp(): void {
    echo "
PHP FFI → JVM Daemon (Shared Memory)
=====================================

Команды:
  exit                      Остановить JVM-демон и выйти

  create <key> <value>      Создать запись
  get <key>                 Получить значение
  set <key> <value>         Обновить значение
  delete <key>              Удалить по ключу
  list                      Список ключей

  help                      Эта справка

";
}

function startDaemon(): void {
    echo "[INFO] Starting JVM daemon...\n";
    $cmd = sprintf('powershell -Command "Start-Process -WindowStyle Hidden java \'-cp output;output\\lib\jna-5.14.0.jar RuntimeStub\'"');
    shell_exec($cmd);
    for ($i = 0; $i < 30; $i++) {
        if (@file_exists('mylang_shm.dat')) {
            echo "[OK] Daemon started\n";
            @file_put_contents('daemon_pid.txt', '');
            return;
        }
        usleep(200000);
    }
    echo "[ERR] Failed to start JVM daemon\n";
    echo "  Try: java -cp \"output;output/lib/jna-5.14.0.jar\" RuntimeStub\n";
    exit(1);
}

function connect(): ?SHMClient {
    try {
        return new SHMClient();
    } catch (Exception $e) {
        return null;
    }
}

function shmIsStale(): bool {
    if (!@file_exists('mylang_shm.dat')) return false;
    // stale if no java process running
    $out = shell_exec('tasklist /NH /FI "IMAGENAME eq java.exe" 2>NUL');
    return $out === null || trim($out) === '' || !str_contains($out, 'java');
}

function main(): void {
    echo "=== PHP FFI → JVM Daemon ===\n\n";

    if (shmIsStale()) {
        @unlink('mylang_shm.dat');
        echo "[INFO] Removed stale SHM file\n";
    }

    // Always start daemon if SHM file doesn't exist
    if (!@file_exists('mylang_shm.dat')) {
        startDaemon();
    }

    $shm = connect();
    if ($shm === null) {
        echo "[ERR] Failed to connect to JVM daemon\n";
        echo "  Try: java -cp \"output;output/lib/jna-5.14.0.jar\" RuntimeStub\n";
        exit(1);
    }
    echo "[OK] Connected\n\n";

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
                    $shm->shutdown();
                    echo "Bye!\n";
                    break 2;

                case 'start':
                    startDaemon();
                    break;

                case 'help':
                    printHelp();
                    break;

                case 'create':
                    $name = array_shift($parts) ?? '';
                    $value = implode(' ', $parts);
                    $shm->create($name, $value);
                    echo "  ok\n";
                    break;

                case 'get':
                    $name = array_shift($parts) ?? '';
                    $r = $shm->get($name);
                    echo "  value: " . ($r ?? '(empty)') . "\n";
                    break;

                case 'set':
                    $name = array_shift($parts) ?? '';
                    $value = implode(' ', $parts);
                    $shm->set($name, $value);
                    echo "  ok\n";
                    break;

                case 'delete':
                    $name = array_shift($parts) ?? '';
                    $shm->delete($name);
                    echo "  ok\n";
                    break;

                case 'list':
                    $r = $shm->list();
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
                    echo "Unknown command: $cmd (type 'help')\n";
            }
        } catch (Exception $e) {
            echo "  ERROR: " . $e->getMessage() . "\n";
        }
    }

    $shm->close();
}

main();

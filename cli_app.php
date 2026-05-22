<?php

require_once __DIR__ . '/shm_client.php';

function printHelp(): void {
    echo "
PHP FFI → JVM Daemon (Shared Memory)
=====================================

Команды:
  start                     Запустить JVM-демон (фоновый процесс)
  exit                      Остановить JVM-демон и выйти

  create <key> <value>      Создать запись
  get <key>                 Получить значение
  set <key> <value>         Обновить значение
  delete <key>              Удалить по ключу
  list                      Список ключей

  exec <func> <args...>     Выполнить builtin-функцию
                              Пример: exec square 7

  help                      Эта справка

";
}

function startDaemon(): void {
    echo "[INFO] Starting JVM daemon...\n";
    $proc = proc_open(
        'START /B java -cp output MainServer',
        [0 => ['pipe', 'r'], 1 => ['pipe', 'w'], 2 => ['pipe', 'w']],
        $pipes
    );
    if (is_resource($proc)) {
        fclose($pipes[0]);
        fclose($pipes[1]);
        fclose($pipes[2]);
        proc_close($proc);
    }
    for ($i = 0; $i < 20; $i++) {
        if (@file_exists('mylang_shm.dat')) {
            echo "[OK] Daemon started\n";
            return;
        }
        usleep(200000);
    }
    echo "[WARN] Daemon may not be ready yet — try connecting anyway.\n";
}

function connect(): ?SHMClient {
    try {
        return new SHMClient();
    } catch (Exception $e) {
        return null;
    }
}

function main(): void {
    $argv = $_SERVER['argv'] ?? [];
    if (isset($argv[1]) && $argv[1] === 'start') {
        startDaemon();
        exit(0);
    }

    echo "=== PHP FFI → JVM Daemon ===\n\n";

    $shm = connect();
    if ($shm === null) {
        echo "[INFO] JVM daemon not running.\n";
        echo "  Run: php cli_app.php start\n";
        echo "  Or:  run_daemon.bat\n";
        exit(1);
    }
    echo "[OK] Connected\n\n";

    while (true) {
        echo "> ";
        $line = trim(fgets(STDIN));
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

                case 'exec':
                    $func = array_shift($parts) ?? '';
                    $args = implode(',', $parts);
                    $r = $shm->exec($func, $args);
                    echo "  result: " . ($r ?? '(empty)') . "\n";
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

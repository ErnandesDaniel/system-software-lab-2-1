<?php

require_once __DIR__ . '/shm_client.php';

function printHelp(): void {
    echo "
PHP FFI → JVM Daemon (Shared Memory)
=====================================

Команды:
  exec <func> <args...>     Выполнить builtin-функцию
                              Пример: exec square 7

  create <key> <value>      Создать запись
  get <key>                 Получить значение
  set <key> <value>         Обновить значение
  delete <key>              Удалить по ключу
  list                      Список ключей

  help                      Эта справка
  exit                      Завершить демон

";
}

function main(): void {
    echo "=== PHP FFI → JVM Daemon ===\n\n";

    $shm = null;
    try {
        $shm = new SHMClient();
        echo "[OK] Connected\n\n";
    } catch (Exception $e) {
        echo "[ERR] " . $e->getMessage() . "\n";
        echo "  run: java -cp output MainServer\n";
        exit(1);
    }

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
                    $shm->exec('exit', '');
                    echo "Bye!\n";
                    break 2;

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

<?php
/**
 * JVM Runner - PHP обертка для запуска JVM функций
 * 
 * Позволяет вызывать функции из скомпилированного MyLang кода на JVM
 */

class JVMRunner {
    private string $classpath;
    private string $runnerClass;
    
    public function __construct(string $classpath = 'output', string $runnerClass = 'MainRunner') {
        $this->classpath = $classpath;
        $this->runnerClass = $runnerClass;
    }
    
    /**
     * Вызвать функцию из JVM с аргументами
     * 
     * @param string $functionName Имя функции (например, 'main', 'square')
     * @param array $args Аргументы функции
     * @return array ['output' => string, 'exit_code' => int, 'success' => bool]
     */
    public function call(string $functionName, array $args = []): array {
        $argsStr = implode(' ', array_map('escapeshellarg', $args));
        $command = sprintf(
            'java -cp %s %s %s %s 2>&1',
            escapeshellarg($this->classpath),
            escapeshellarg($this->runnerClass),
            escapeshellarg($functionName),
            $argsStr
        );
        
        $output = [];
        $exitCode = 0;
        exec($command, $output, $exitCode);
        
        return [
            'output' => implode("\n", $output),
            'exit_code' => $exitCode,
            'success' => $exitCode === 0
        ];
    }
    
    /**
     * Вызвать функцию и получить только вывод
     */
    public function callSimple(string $functionName, array $args = []): string {
        $result = $this->call($functionName, $args);
        return $result['output'];
    }
    
    /**
     * Вызвать функцию и получить результат как число
     */
    public function callInt(string $functionName, array $args = []): ?int {
        $result = $this->call($functionName, $args);
        if (!$result['success']) {
            return null;
        }
        
        // Последняя строка обычно содержит возвращаемое значение
        $lines = explode("\n", trim($result['output']));
        $lastLine = end($lines);
        
        return is_numeric($lastLine) ? (int)$lastLine : null;
    }
}

// === Примеры использования ===

echo "=== JVM Runner from PHP ===\n\n";

// Создаем экземпляр
// Используем полный путь или путь относительно текущей директории
$classpath = __DIR__ . '/output';
$jvm = new JVMRunner($classpath);

// Пример 1: Запуск main функции
echo "1. Запуск main():\n";
$result = $jvm->call('main');
echo "Вывод:\n" . $result['output'] . "\n";

// Пример 5: Несколько вызовов
$numbers = [3, 5, 7, 10];
echo "5. Таблица квадратов:\n";
foreach ($numbers as $n) {
    $result = $jvm->callInt('square', [(string)$n]);
    echo "  square($n) = $result\n";
}

echo "\n=== Готово! ===\n";

<?php
require __DIR__ . '/shm_client.php';

$c = new SHMClient();

$tests = [
    'exec square 7'      => fn() => $c->exec('square', '7'),
    'create note1 Hello' => fn() => $c->create('note1', 'Hello World'),
    'create note2 PHPJVM'=> fn() => $c->create('note2', 'PHP+JVM via SHM'),
    'list'               => fn() => $c->list(),
    'get note1'          => fn() => $c->get('note1'),
    'set note1 Updated'  => fn() => $c->set('note1', 'Updated Value'),
    'get note1'          => fn() => $c->get('note1'),
    'delete note2'       => fn() => $c->delete('note2'),
    'list'               => fn() => $c->list(),
];

foreach ($tests as $label => $fn) {
    echo ">>> $label\n";
    try {
        $r = $fn();
        echo "  ok" . ($r !== null ? ": $r" : "") . "\n";
    } catch (Exception $e) {
        echo "  ERROR: " . $e->getMessage() . "\n";
    }
    echo "\n";
}

$c->close();
echo "Done.\n";

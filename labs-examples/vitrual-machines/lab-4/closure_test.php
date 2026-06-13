<?php

function startMyLang(): array {
    $cmd = 'java -cp "output" RuntimeStub';
    $descriptors = [
        0 => ['pipe', 'r'],
        1 => ['pipe', 'w'],
        2 => ['pipe', 'w'],
    ];
    $process = proc_open($cmd, $descriptors, $pipes, getcwd());
    if (!is_resource($process)) {
        throw new RuntimeException("Failed to start MyLang");
    }
    return [$process, $pipes];
}

function sendCmd($stdin, $stdout, string $cmd): ?int {
    fwrite($stdin, $cmd . "\n");
    fflush($stdin);

    $response = '';
    $start = microtime(true);
    stream_set_blocking($stdout, false);
    while (microtime(true) - $start < 5.0) {
        $chunk = fread($stdout, 4096);
        if ($chunk !== false && $chunk !== '') {
            $response .= $chunk;
            if (str_contains($response, "\n")) { break; }
        }
        usleep(50000);
    }
    $response = rtrim(explode("\n", trim($response))[0]);

    if (str_starts_with($response, 'ERR ')) {
        throw new RuntimeException(substr($response, 4));
    }
    if (str_starts_with($response, 'OK')) {
        $payload = trim(substr($response, 2));
        return $payload !== '' ? (int)$payload : null;
    }
    throw new RuntimeException("Bad response: $response");
}


/*
$x = ....;

$c1 = $x['make_pair']->call();
$c2 = $x['make_pair']->call();

        printf("c1[0](2) = %d\n", $c1->getItem(0)->call(2));

*/
function test(): int {
    [$process, $pipes] = startMyLang();
    [$stdin, $stdout, $stderr] = $pipes;

    try {
        sendCmd($stdin, $stdout, 'new 0');
        sendCmd($stdin, $stdout, 'new 1');

        $x1 = sendCmd($stdin, $stdout, 'call 0 0 2');
        printf("c1[0](2) = %d\n", $x1);

        $y1 = sendCmd($stdin, $stdout, 'call 1 0 2');
        printf("c2[0](2) = %d\n", $y1);

        $x2 = sendCmd($stdin, $stdout, 'call 0 0 2');
        printf("c1[0](2) = %d\n", $x2);

        $y2 = sendCmd($stdin, $stdout, 'call 1 1 7');
        printf("c2[1](7) = %d\n", $y2);

        $x3 = sendCmd($stdin, $stdout, 'call 0 1 7');
        printf("c1[1](7) = %d\n", $x3);

        $y3 = sendCmd($stdin, $stdout, 'call 1 0 3');
        printf("c2[0](3) = %d\n", $y3);

        sendCmd($stdin, $stdout, 'exit');

        return ($x1 + $x2 + $x3) * 100 + ($y1 + $y2 + $y3);
    } finally {
        fclose($stdin);
        fclose($stdout);
        fclose($stderr);
        proc_close($process);
    }
}

$r = test();
printf("test() = %d\n", $r);

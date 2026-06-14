<?php

class MyLangSession {
    private $process;
    private $stdin;
    private $stdout;
    private $nextId = 0;

    public function __construct() {
        $cmd = 'java -cp "output" RuntimeStub';
        $descriptors = [
            0 => ['pipe', 'r'],
            1 => ['pipe', 'w'],
            2 => ['pipe', 'w'],
        ];
        $this->process = proc_open($cmd, $descriptors, $pipes, getcwd());
        if (!is_resource($this->process)) {
            throw new RuntimeException("Failed to start MyLang");
        }
        [$this->stdin, $this->stdout, $stderr] = $pipes;
    }

    public function __destruct() {
        @fclose($this->stdin);
        @fclose($this->stdout);
        if (is_resource($this->process)) {
            proc_close($this->process);
        }
    }

    public function makePair(): Pair {
        $id = $this->nextId++;
        $this->sendRaw("new $id");
        return new Pair($id, $this);
    }

    public function sendRaw(string $cmd): ?int {
        fwrite($this->stdin, $cmd . "\n");
        fflush($this->stdin);

        $response = '';
        $start = microtime(true);
        stream_set_blocking($this->stdout, false);
        while (microtime(true) - $start < 5.0) {
            $chunk = fread($this->stdout, 4096);
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
}

class Pair {
    private $id;
    private $session;

    public function __construct(int $id, MyLangSession $session) {
        $this->id = $id;
        $this->session = $session;
    }

    public function getItem(int $idx): MyLangClosure {
        return new MyLangClosure($this->id, $idx, $this->session);
    }
}

class MyLangClosure {
    private $id;
    private $idx;
    private $session;

    public function __construct(int $id, int $idx, MyLangSession $session) {
        $this->id = $id;
        $this->idx = $idx;
        $this->session = $session;
    }

    public function call(int $x): int {
        return $this->session->sendRaw("call $this->id $this->idx $x");
    }
}

function test(): int {
    $x = new MyLangSession();

    $c1 = $x->makePair();
    $c2 = $x->makePair();

    $x1 = $c1->getItem(0)->call(2);
    printf("c1[0](2) = %d\n", $x1);

    $y1 = $c2->getItem(0)->call(2);
    printf("c2[0](2) = %d\n", $y1);

    $x2 = $c1->getItem(0)->call(2);
    printf("c1[0](2) = %d\n", $x2);

    $y2 = $c2->getItem(1)->call(7);
    printf("c2[1](7) = %d\n", $y2);

    $x3 = $c1->getItem(1)->call(7);
    printf("c1[1](7) = %d\n", $x3);

    $y3 = $c2->getItem(0)->call(3);
    printf("c2[0](3) = %d\n", $y3);

    $x->sendRaw('exit');

    return ($x1 + $x2 + $x3) * 100 + ($y1 + $y2 + $y3);
}

$r = test();
printf("test() = %d\n", $r);

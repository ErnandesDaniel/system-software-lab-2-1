import socket, re, time

def test():
    s = socket.socket(); s.settimeout(10)
    s.connect(('127.0.0.1', 2121))
    print('GOT:', s.recv(4096).decode().strip())
    
    def cmd(c):
        s.sendall(c)
        return s.recv(4096).decode().strip()
    
    print('USER:', cmd(b'USER ftp\r\n'))
    print('PASS:', cmd(b'PASS test\r\n'))
    print('SYST:', cmd(b'SYST\r\n'))
    print('PWD:', cmd(b'PWD\r\n'))
    
    # Test EPSV
    print('EPSV:', cmd(b'EPSV\r\n'))
    
    # Test PASV + LIST
    r = cmd(b'PASV\r\n')
    print('PASV:', r)
    m = re.search(r'\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)', r)
    dp = int(m.group(5))*256 + int(m.group(6))
    ds = socket.socket(); ds.settimeout(10); ds.connect(('127.0.0.1', dp))
    r = cmd(b'LIST\r\n')
    print('LIST:', r)
    time.sleep(0.3)
    ld = b''
    while True:
        try: c = ds.recv(4096)
        except: break
        if not c: break
        ld += c
    ds.close()
    print('Data:', ld.decode())
    r226 = s.recv(4096).decode().strip()
    print('226:', r226)
    
    # RETR a.txt
    r = cmd(b'PASV\r\n')
    print('PASV2:', r)
    m = re.search(r'\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)', r)
    dp = int(m.group(5))*256 + int(m.group(6))
    
    # Test SIZE first
    print('SIZE:', cmd(b'SIZE a.txt\r\n'))
    
    ds = socket.socket(); ds.settimeout(10); ds.connect(('127.0.0.1', dp))
    r = cmd(b'RETR a.txt\r\n')
    print('RETR:', r)
    time.sleep(0.3)
    fd = b''
    while True:
        try: c = ds.recv(4096)
        except: break
        if not c: break
        fd += c
    ds.close()
    print('File:', repr(fd))
    r226 = s.recv(4096).decode().strip()
    print('226:', r226)
    
    print('QUIT:', cmd(b'QUIT\r\n'))
    s.close()
    print('ALL OK')

test()

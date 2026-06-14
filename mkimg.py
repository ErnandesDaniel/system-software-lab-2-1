import struct
bs=1024;bt=256;it=32;isz=128
img=bytearray(bt*bs)
sb=memoryview(img)[1024:2048]
struct.pack_into('<III',sb,0,it,bt,0)
struct.pack_into('<I',sb,20,1)
struct.pack_into('<II',sb,24,0,bt)
struct.pack_into('<I',sb,40,it)
struct.pack_into('<H',sb,56,0xEF53)
struct.pack_into('<H',sb,88,isz)
bg=memoryview(img)[2048:2080]
struct.pack_into('<III',bg,0,3,4,5)
struct.pack_into('<HH',bg,12,bt-12-1,it-5)
struct.pack_into('<H',bg,16,3)
for b in range(13): img[3072+b//8]|=1<<(b%8)
for i in[1,2,12,13,14]: img[4096+i//8]|=1<<(i%8)
def wi(t,inum,m,s,b0,l):
    o=(inum-1)*128
    struct.pack_into('<H',t,o,m)
    struct.pack_into('<I',t,o+4,s)
    struct.pack_into('<H',t,o+26,l)
    struct.pack_into('<I',t,o+40,b0)
itbl=memoryview(img)[5120:5120+it*isz]
wi(itbl,2,0x41ED,bs,9,2)
wi(itbl,12,0x41ED,bs,10,2)
wi(itbl,13,0x81A4,6,11,1)
wi(itbl,14,0x81A4,5,12,1)
def wd(d,o,ino,r,nl,ft,nm):
    struct.pack_into('<I',d,o,ino)
    struct.pack_into('<H',d,o+4,r)
    d[o+6]=nl; d[o+7]=ft
    for i,b in enumerate(nm): d[o+8+i]=b
rd=memoryview(img)[9*bs:]
wd(rd,0,2,12,1,2,b'.')
wd(rd,12,2,12,2,2,b'..')
wd(rd,24,12,16,6,2,b'subdir')
wd(rd,40,13,984,5,1,b'a.txt')
sd=memoryview(img)[10*bs:]
wd(sd,0,12,12,1,2,b'.')
wd(sd,12,2,12,2,2,b'..')
wd(sd,24,14,1000,5,1,b'b.txt')
img[11*bs:11*bs+6]=b'Hello\n'
img[12*bs:12*bs+5]=b'Data\n'
with open('/tmp/test_ext3.img','wb') as f: f.write(img)
print('OK')

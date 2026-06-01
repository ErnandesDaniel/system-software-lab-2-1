use super::compile_and_run;
use super::compile_and_run_with_files;

#[test]
fn test_exe_puts() {
    let output = compile_and_run("import puts def main() puts(\"hello\") return 0 end");
    assert!(output.status.success(), "puts should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).contains("hello"), "should print hello");
}

#[test]
fn test_exe_putchar() {
    let source = "import putchar def main() putchar(65) putchar(66) return 0 end";
    let output = compile_and_run(source);
    assert!(output.status.success(), "putchar should succeed");
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("AB"), "should print AB, got: {}", out);
}

#[test]
fn test_exe_strlen() {
    let source = "import strlen def main() return strlen(\"hello\") end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(5), "strlen(\"hello\") = 5");
}

#[test]
fn test_exe_strcpy_strcat() {
    let source = r#"
import strcpy
import strcat
import puts
global buf of string = "                                ";
def main() of int
    strcpy(buf, "hello ")
    strcat(buf, "world")
    puts(buf)
    return 0
end
"#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "strcpy+strcat should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).contains("hello world"));
}

#[test]
fn test_exe_strcmp_eq() {
    let source = "import strcmp def main() if strcmp(\"abc\", \"abc\") == 0 then return 1 else return 0 end end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "strcmp equal should return 0→true");
}

#[test]
fn test_exe_strcmp_gt() {
    let source = "import strcmp def main() if strcmp(\"b\", \"a\") > 0 then return 1 else return 0 end end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "strcmp b>a should be >0");
}

#[test]
fn test_exe_strcmp_lt() {
    let source = "import strcmp def main() if strcmp(\"a\", \"b\") < 0 then return 1 else return 0 end end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "strcmp a<b should be <0");
}

#[test]
fn test_exe_strchr() {
    let source = r#"
import strchr
import strlen
def main() of int
    s = "hello"
    p = strchr(s, 101)
    if p == "" then return 99 end
    return strlen(p)
end
"#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(4), "strchr('hello','e') → 'ello', len=4");
}

#[test]
fn test_exe_atoi() {
    let source = "import atoi def main() return atoi(\"42\") end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "atoi(\"42\") = 42");
}

#[test]
fn test_exe_malloc_free() {
    let source = r#"
import malloc
import free
def main() of int
    p = malloc(128)
    if p == "" then return 1 end
    free(p)
    return 0
end
"#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "malloc+free should return 0");
}

#[test]
fn test_exe_memcpy() {
    let source = r#"
import memcpy
import puts
global buf of string = "                                ";
def main() of int
    memcpy(buf, "OK", 2)
    puts(buf)
    return 0
end
"#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "memcpy should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("OK"));
}

#[test]
fn test_exe_sprintf() {
    let source = r#"
import sprintf
import puts
global buf of string = "                                ";
def main() of int
    sprintf(buf, "%d", 12345)
    puts(buf)
    return 0
end
"#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "sprintf should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).contains("12345"));
}

#[test]
fn test_exe_fopen_fclose() {
    let source = r#"
import fopen
import fclose
global s of string;
def main() of int
    f = fopen("_test_fopen.txt", "w")
    if f == "" then puts("FAIL") else puts("OK") end
    if f == "" then return 1 end
    fclose(f)
    return 0
end
"#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "fopen+fclose should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).contains("OK"));
}

#[test]
fn test_exe_fopen_read_fgetc() {
    let source = r#"
import fopen
import fgetc
import fclose
def main() of int
    f = fopen("_test_fopen.txt", "r")
    c = fgetc(f)
    putchar(c)
    c = fgetc(f)
    putchar(c)
    fclose(f)
    return 0
end
"#;
    let output = compile_and_run_with_files(source, &[("_test_fopen.txt", "AB")]);
    assert!(output.status.success(), "fopen+fgetc should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).contains("AB"));
}

#[test]
fn test_exe_feof() {
    let source = r#"
import fopen
import fclose
import fgetc
import feof
import puts
def main() of int
    f = fopen("test.txt", "r")
    c = fgetc(f)
    c = fgetc(f)
    c = fgetc(f)
    e = feof(f)
    if e != 0 then puts("EOF") else puts("NOT") end
    fclose(f)
    return 0
end
"#;
    let output = compile_and_run_with_files(source, &[("test.txt", "AB")]);
    assert!(output.status.success(), "feof should succeed");
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("EOF"), "expected EOF, got: {}", out);
}

#[test]
fn test_exe_fgets() {
    let source = r#"
import fopen
import fclose
import fgets
import puts
global buf of string = "                                ";
def main() of int
    f = fopen("test.txt", "r")
    fgets(buf, 64, f)
    puts(buf)
    fclose(f)
    return 0
end
"#;
    let output = compile_and_run_with_files(source, &[("test.txt", "hello world")]);
    assert!(output.status.success(), "fgets should succeed");
    assert!(String::from_utf8_lossy(&output.stdout).contains("hello world"));
}

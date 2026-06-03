use crate::codegen::jvm::types::capitalize_first;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrFunction, IrType};
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Field, FieldAccessFlags, FieldType, Method, MethodAccessFlags};

const COROUTINE_MAX_STACK: u16 = 4;
const COROUTINE_MAX_LOCALS: u16 = 3;

impl JvmGenerator {
    pub(super) fn generate_runtime_stub(&mut self, functions: &[IrFunction]) -> Vec<u8> {
        let this_class = self.pool.constant_pool.add_class("RuntimeStub")
            .expect("Failed to add RuntimeStub class");
        let super_class = self.pool.constant_pool.add_class("java/lang/Object")
            .expect("Failed to add Object class");
        let code_attr = self.pool.constant_pool.add_utf8("Code")
            .expect("Failed to add 'Code' UTF8");
        let coro_field_name = self.pool.constant_pool.add_utf8("coroutines")
            .expect("Failed to add 'coroutines' UTF8");
        let coro_field_desc = self.pool.constant_pool.add_utf8("[Ljava/lang/Object;")
            .expect("Failed to add coro field descriptor");
        let coro_field_ref = self.pool.constant_pool.add_field_ref(this_class, "coroutines", "[Ljava/lang/Object;")
            .expect("Failed to add coroutines field ref");

        let coro_info: Vec<(String, u16)> = functions.iter()
            .filter(|f| f.is_coroutine)
            .map(|f| {
                let name = if f.name == "main" { "Main".to_string() } else { capitalize_first(&f.name) };
                let class_idx = self.pool.constant_pool.add_class(&name)
                    .expect("Failed to add coroutine class");
                let _ = self.pool.constant_pool.add_method_ref(class_idx, "<init>", "()V")
                    .expect("Failed to add coroutine init ref");
                let _ = self.pool.constant_pool.add_method_ref(class_idx, "resume", "()I")
                    .expect("Failed to add coroutine resume ref");
                (name, class_idx)
            }).collect();
        let count = coro_info.len();
        let mut methods = Vec::new();

        if count > 0 {
            self.build_coro_init(&mut methods, &coro_info, count, code_attr, coro_field_ref);
            self.build_coro_resume(&mut methods, &coro_info, code_attr, coro_field_ref);
            self.build_coro_get_state(&mut methods, &coro_info, code_attr, coro_field_ref);
            self.build_coro_set_param(&mut methods, &coro_info, code_attr, coro_field_ref);
        }

        let putchar_name = self.pool.constant_pool.add_utf8("putchar")
            .expect("Failed to add 'putchar' UTF8");
        let putchar_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("Failed to add putchar descriptor");
        let system_class = self.pool.constant_pool.add_class("java/lang/System")
            .expect("Failed to add System class");
        let system_out_ref = self.pool.constant_pool.add_field_ref(system_class, "out", "Ljava/io/PrintStream;")
            .expect("Failed to add System.out field ref");
        let printstream_class = self.pool.constant_pool.add_class("java/io/PrintStream")
            .expect("Failed to add PrintStream class");
        let print_char_ref = self.pool.constant_pool.add_method_ref(printstream_class, "print", "(C)V")
            .expect("Failed to add print method ref");
        let println_string_ref = self.pool.constant_pool.add_method_ref(printstream_class, "println", "(Ljava/lang/String;)V")
            .expect("Failed to add println method ref");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: putchar_name,
            descriptor_index: putchar_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 2,
                max_locals: 1,
                code: vec![
                    Instruction::Getstatic(system_out_ref),
                    Instruction::Iload_0,
                    Instruction::I2c,
                    Instruction::Invokevirtual(print_char_ref),
                    Instruction::Iload_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let string_class = self.pool.constant_pool.add_class("java/lang/String")
            .expect("Failed to add String class");
        let string_byte_init = self.pool.constant_pool.add_method_ref(string_class, "<init>", "([B)V")
            .expect("Failed to add String(byte[]) init");

        // === File I/O static fields (moved before puts) ===
        let file_fds_name = self.pool.constant_pool.add_utf8("fileStreams").expect("utf8");
        let file_fds_desc = self.pool.constant_pool.add_utf8("[Ljava/io/InputStream;").expect("utf8");
        let file_next_name = self.pool.constant_pool.add_utf8("fileNext").expect("utf8");
        let file_next_desc = self.pool.constant_pool.add_utf8("I").expect("utf8");
        let file_fds_ref = self.pool.constant_pool.add_field_ref(this_class, "fileStreams", "[Ljava/io/InputStream;").expect("fref");
        let file_next_ref = self.pool.constant_pool.add_field_ref(this_class, "fileNext", "I").expect("fref");

        // === I/O class refs ===
        let string_class = self.pool.constant_pool.add_class("java/lang/String").expect("class");
        let str_init = self.pool.constant_pool.add_method_ref(string_class, "<init>", "([B)V").expect("mref");
        let fis_class = self.pool.constant_pool.add_class("java/io/FileInputStream").expect("class");
        let fis_init = self.pool.constant_pool.add_method_ref(fis_class, "<init>", "(Ljava/lang/String;)V").expect("mref");
        let is_class = self.pool.constant_pool.add_class("java/io/InputStream").expect("class");
        let is_read = self.pool.constant_pool.add_method_ref(is_class, "read", "()I").expect("mref");
        let is_close = self.pool.constant_pool.add_method_ref(is_class, "close", "()V").expect("mref");
        let int_class = self.pool.constant_pool.add_class("java/lang/Integer").expect("class");
        let int_parse = self.pool.constant_pool.add_method_ref(int_class, "parseInt", "(Ljava/lang/String;)I").expect("mref");
        let str_init_3arg = self.pool.constant_pool.add_method_ref(string_class, "<init>", "([BII)V").expect("mref");
        // strlen([B)I = scan for null byte only
        let strlen_ref = self.pool.constant_pool.add_method_ref(this_class, "strlen", "([B)I").expect("mref");
        // Also add strlen using nullscan's local count setup but with different check

        // === fopen([B[B)I ===
        // byte[] → String (strip null terminator), then open FileInputStream
        let fopen_name = self.pool.constant_pool.add_utf8("fopen").expect("utf8");
        let fopen_desc = self.pool.constant_pool.add_utf8("([B[B)I").expect("utf8");
        let nullscan_name = self.pool.constant_pool.add_utf8("nullscan").expect("utf8");
        let nullscan_desc = self.pool.constant_pool.add_utf8("([B)I").expect("utf8");
        // nullscan helper: scan for null byte only (stops at 0)
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: nullscan_name,
            descriptor_index: nullscan_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 3, max_locals: 3,
                code: vec![
                    Instruction::Iconst_0,
                    Instruction::Istore(1),
                    Instruction::Goto(4),
                    Instruction::Iinc(1, 1),
                    Instruction::Iload(1),
                    Instruction::Aload_0,
                    Instruction::Arraylength,
                    Instruction::If_icmpge(13),
                    Instruction::Aload_0,
                    Instruction::Iload(1),
                    Instruction::Baload,
                    Instruction::Ifeq(13),
                    Instruction::Goto(3),
                    Instruction::Iload(1),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
        self.pool.nullscan_ref = self.pool.constant_pool.add_method_ref(this_class, "nullscan", "([B)I").expect("mref");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: fopen_name,
            descriptor_index: fopen_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5, max_locals: 4,
                code: vec![
                    // Call nullscan(path) to get length, then new String(path, 0, len)
                    Instruction::Aload_0,
                    Instruction::Invokestatic(self.pool.nullscan_ref),
                    Instruction::Istore(3),
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Iconst_0,
                    Instruction::Iload(3),
                    Instruction::Invokespecial(str_init_3arg),
                    Instruction::Astore(2),
                    // new FileInputStream(str)
                    Instruction::New(fis_class),
                    Instruction::Dup,
                    Instruction::Aload_2,
                    Instruction::Invokespecial(fis_init),
                    Instruction::Astore_2,
                    // fileStreams[fileNext] = fis
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Getstatic(file_next_ref),
                    Instruction::Aload_2,
                    Instruction::Aastore,
                    // return fileNext++ (add 1 so 0 = error)
                    Instruction::Getstatic(file_next_ref),
                    Instruction::Dup,
                    Instruction::Iconst_1,
                    Instruction::Iadd,
                    Instruction::Putstatic(file_next_ref),
                    Instruction::Iconst_1,
                    Instruction::Iadd,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === puts([B)I ===
        let puts_name = self.pool.constant_pool.add_utf8("puts").expect("utf8");
        let puts_desc = self.pool.constant_pool.add_utf8("([B)I").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: puts_name,
            descriptor_index: puts_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 6, max_locals: 2,
                code: vec![
                    Instruction::Getstatic(system_out_ref),
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Iconst_0,
                    Instruction::Aload_0,
                    Instruction::Invokestatic(self.pool.nullscan_ref),
                    Instruction::Invokespecial(str_init_3arg),
                    Instruction::Invokevirtual(println_string_ref),
                    Instruction::Iconst_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === fgetc(I)I ===
        let fgetc_name = self.pool.constant_pool.add_utf8("fgetc").expect("utf8");
        let fgetc_desc = self.pool.constant_pool.add_utf8("(I)I").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: fgetc_name,
            descriptor_index: fgetc_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 2, max_locals: 2,
                code: vec![
                    // handle = arg - 1 (since fopen returns handle+1)
                    Instruction::Iload_0,
                    Instruction::Iconst_1,
                    Instruction::Isub,
                    Instruction::Istore(1),
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Iload(1),
                    Instruction::Aaload,
                    Instruction::Checkcast(is_class),
                    Instruction::Invokevirtual(is_read),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === fclose(I)I ===
        let fclose_name = self.pool.constant_pool.add_utf8("fclose").expect("utf8");
        let fclose_desc = self.pool.constant_pool.add_utf8("(I)I").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: fclose_name,
            descriptor_index: fclose_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 3, max_locals: 2,
                code: vec![
                    // handle = arg - 1
                    Instruction::Iload_0,
                    Instruction::Iconst_1,
                    Instruction::Isub,
                    Instruction::Istore(1),
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Iload(1),
                    Instruction::Aaload,
                    Instruction::Checkcast(is_class),
                    Instruction::Invokevirtual(is_close),
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Iload(1),
                    Instruction::Aconst_null,
                    Instruction::Aastore,
                    Instruction::Iconst_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === malloc(I)[B ===
        let malloc_name = self.pool.constant_pool.add_utf8("malloc").expect("utf8");
        let malloc_desc = self.pool.constant_pool.add_utf8("(I)[B").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: malloc_name,
            descriptor_index: malloc_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 1, max_locals: 1,
                code: vec![
                    Instruction::Iload_0,
                    Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte),
                    Instruction::Areturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === free([B)V ===
        let free_name = self.pool.constant_pool.add_utf8("free").expect("utf8");
        let free_desc = self.pool.constant_pool.add_utf8("([B)V").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: free_name,
            descriptor_index: free_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 0, max_locals: 1,
                code: vec![
                    Instruction::Return,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === string_slice([BII)[B ===
        let ss_name = self.pool.constant_pool.add_utf8("string_slice").expect("utf8");
        let ss_desc = self.pool.constant_pool.add_utf8("([BII)[B").expect("utf8");
        let system_class = self.pool.constant_pool.add_class("java/lang/System").expect("class");
        let arraycopy_ref = self.pool.constant_pool.add_method_ref(system_class, "arraycopy", "(Ljava/lang/Object;ILjava/lang/Object;II)V").expect("mref");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: ss_name,
            descriptor_index: ss_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5, max_locals: 7,
                code: vec![
                    // len = end - start (simple version, no null scanning)
                    Instruction::Iload_2,
                    Instruction::Iload_1,
                    Instruction::Isub,
                    Instruction::Dup,
                    Instruction::Istore(4),
                    // byte[] result = new byte[len]
                    Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte),
                    Instruction::Astore(3),
                    // System.arraycopy(arr, start, result, 0, len)
                    Instruction::Aload_0,
                    Instruction::Iload_1,
                    Instruction::Aload(3),
                    Instruction::Iconst_0,
                    Instruction::Iload(4),
                    Instruction::Invokestatic(arraycopy_ref),
                    // return result
                    Instruction::Aload(3),
                    Instruction::Areturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === printf helpers ===
        let print_str_ref = self.pool.constant_pool.add_method_ref(printstream_class, "print", "(Ljava/lang/String;)V").expect("mref");
        let int_to_str = self.pool.constant_pool.add_method_ref(int_class, "toString", "(I)Ljava/lang/String;").expect("mref");
        let str_replace = self.pool.constant_pool.add_method_ref(string_class, "replaceFirst", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;").expect("mref");

        // Helper: build code to create String from byte[] with nullscan
        macro_rules! emit_fmt_string {
            ($code:expr) => {
                $code.push(Instruction::New(string_class));
                $code.push(Instruction::Dup);
                $code.push(Instruction::Aload_0);
                $code.push(Instruction::Iconst_0);
                $code.push(Instruction::Aload_0);
                $code.push(Instruction::Invokestatic(self.pool.nullscan_ref));
                $code.push(Instruction::Invokespecial(str_init_3arg));
            };
        }

        // printf([BI)I — print format with %d replaced by value
        let printf_name = self.pool.constant_pool.add_utf8("printf").expect("utf8");
        let printf1_desc = self.pool.constant_pool.add_utf8("([BI)I").expect("utf8");
        let mut c1 = Vec::new();
        // Build "%d" byte[] FIRST
        c1.push(Instruction::Iconst_2);
        c1.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte));
        c1.push(Instruction::Dup);
        c1.push(Instruction::Iconst_0);
        c1.push(Instruction::Bipush(37)); // '%'
        c1.push(Instruction::Bastore);
        c1.push(Instruction::Dup);
        c1.push(Instruction::Iconst_1);
        c1.push(Instruction::Bipush(100)); // 'd'
        c1.push(Instruction::Bastore);
        c1.push(Instruction::Astore(2)); // local 2 = byte[2] "%d"
        // String fmt = new String(bytes, 0, nullscan(bytes))
        c1.push(Instruction::New(string_class));
        c1.push(Instruction::Dup);
        c1.push(Instruction::Aload_0);
        c1.push(Instruction::Iconst_0);
        c1.push(Instruction::Aload_0);
        c1.push(Instruction::Invokestatic(self.pool.nullscan_ref));
        c1.push(Instruction::Invokespecial(str_init_3arg));
        c1.push(Instruction::Astore(3)); // local 3 = fmt
        // String pctD = new String(byte[2])
        c1.push(Instruction::New(string_class));
        c1.push(Instruction::Dup);
        c1.push(Instruction::Aload(2)); // load byte[] "%d"
        c1.push(Instruction::Invokespecial(string_byte_init));
        c1.push(Instruction::Astore(2)); // local 2 = "%d" string (reuse)
        // String valStr = Integer.toString(val1)
        c1.push(Instruction::Iload_1);
        c1.push(Instruction::Invokestatic(int_to_str));
        c1.push(Instruction::Astore(4)); // local 4 = valStr
        // result = fmt.replace(pctD, valStr)
        c1.push(Instruction::Aload(3));
        c1.push(Instruction::Aload(2));
        c1.push(Instruction::Aload(4));
        c1.push(Instruction::Invokevirtual(str_replace));
        // System.out.print(result)
        c1.push(Instruction::Getstatic(system_out_ref));
        c1.push(Instruction::Swap);
        c1.push(Instruction::Invokevirtual(print_str_ref));
        c1.push(Instruction::Iconst_0);
        c1.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: printf_name,
            descriptor_index: printf1_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5, max_locals: 5,
                code: c1,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // printf([BII)I — print format with two %d replaced by values
        let printf2_desc = self.pool.constant_pool.add_utf8("([BII)I").expect("utf8");
        let mut c2 = Vec::new();
        // Build "%d" byte[] FIRST
        c2.push(Instruction::Iconst_2);
        c2.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte));
        c2.push(Instruction::Dup);
        c2.push(Instruction::Iconst_0);
        c2.push(Instruction::Bipush(37));
        c2.push(Instruction::Bastore);
        c2.push(Instruction::Dup);
        c2.push(Instruction::Iconst_1);
        c2.push(Instruction::Bipush(100));
        c2.push(Instruction::Bastore);
        c2.push(Instruction::Astore(5)); // local 5 = byte[2] "%d" (don't overwrite params!)
        // String fmt = new String(bytes, 0, nullscan(bytes))
        c2.push(Instruction::New(string_class));
        c2.push(Instruction::Dup);
        c2.push(Instruction::Aload_0);
        c2.push(Instruction::Iconst_0);
        c2.push(Instruction::Aload_0);
        c2.push(Instruction::Invokestatic(self.pool.nullscan_ref));
        c2.push(Instruction::Invokespecial(str_init_3arg));
        c2.push(Instruction::Astore(3)); // local 3 = fmt
        // String pctD = new String(byte[5])
        c2.push(Instruction::New(string_class));
        c2.push(Instruction::Dup);
        c2.push(Instruction::Aload(5));
        c2.push(Instruction::Invokespecial(string_byte_init));
        c2.push(Instruction::Astore(5)); // local 5 = "%d" string
        // temp = fmt.replace(pctD, Integer.toString(val1))
        c2.push(Instruction::Aload(3));
        c2.push(Instruction::Aload(5));
        c2.push(Instruction::Iload_1);
        c2.push(Instruction::Invokestatic(int_to_str));
        c2.push(Instruction::Invokevirtual(str_replace));
        c2.push(Instruction::Astore(3)); // local 3 = temp
        // result = temp.replace(pctD, Integer.toString(val2))
        c2.push(Instruction::Aload(3));
        c2.push(Instruction::Aload(5));
        c2.push(Instruction::Iload_2);
        c2.push(Instruction::Invokestatic(int_to_str));
        c2.push(Instruction::Invokevirtual(str_replace));
        // System.out.print(result)
        c2.push(Instruction::Getstatic(system_out_ref));
        c2.push(Instruction::Swap);
        c2.push(Instruction::Invokevirtual(print_str_ref));
        c2.push(Instruction::Iconst_0);
        c2.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: printf_name,
            descriptor_index: printf2_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5, max_locals: 6,
                code: c2,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === atoi([B)I ===
        let atoi_name = self.pool.constant_pool.add_utf8("atoi").expect("utf8");
        let atoi_desc = self.pool.constant_pool.add_utf8("([B)I").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: atoi_name,
            descriptor_index: atoi_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5, max_locals: 3,
                code: vec![
                    // i = 0
                    Instruction::Iconst_0,
                    Instruction::Istore(1),
                    Instruction::Goto(4),
                    // loop body: i++ (index 3)
                    Instruction::Iinc(1, 1),
                    // check (index 4): if i >= arr.length, exit
                    Instruction::Iload(1),
                    Instruction::Aload_0,
                    Instruction::Arraylength,
                    Instruction::If_icmpge(21),
                    // load byte at arr[i]
                    Instruction::Aload_0,
                    Instruction::Iload(1),
                    Instruction::Baload,
                    Instruction::Istore(2),
                    // if byte == 0, exit
                    Instruction::Iload(2),
                    Instruction::Ifeq(21),
                    // if byte == 10, exit
                    Instruction::Iload(2),
                    Instruction::Bipush(10),
                    Instruction::If_icmpeq(21),
                    // if byte == 13, exit
                    Instruction::Iload(2),
                    Instruction::Bipush(13),
                    Instruction::If_icmpeq(21),
                    // else continue loop
                    Instruction::Goto(3),
                    // new String(bytes, 0, i) (index 21)
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Iconst_0,
                    Instruction::Iload(1),
                    Instruction::Invokespecial(str_init_3arg),
                    Instruction::Invokestatic(int_parse),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // === Static initializer: fileStreams + global structs/arrays ===
        let clinit_name = self.pool.constant_pool.add_utf8("<clinit>").expect("utf8");
        let clinit_desc = self.pool.constant_pool.add_utf8("()V").expect("utf8");
        let mut clinit_code = vec![
            Instruction::Bipush(16),
            Instruction::Anewarray(is_class),
            Instruction::Putstatic(file_fds_ref),
        ];
        for (gname, gty) in &self.global.global_vars {
            if let Some(&fr) = self.global.global_field_refs.get(gname) {
                if self.global.global_uses_object_array.contains(gname) {
                    let size = self.get_global_object_array_inner_size(gname) as i8;
                    if self.pool.object_class_idx == 0 {
                        self.pool.object_class_idx = self.pool.constant_pool.add_class("java/lang/Object")
                            .expect("Failed to add Object class");
                    }
                    clinit_code.push(Instruction::Bipush(size));
                    clinit_code.push(Instruction::Anewarray(self.pool.object_class_idx));
                    clinit_code.push(Instruction::Putstatic(fr));
                } else if let IrType::Array(_, n) = gty {
                    if *n > 0 {
                        let sz = *n as i16;
                        if sz <= 127 {
                            clinit_code.push(Instruction::Bipush(sz as i8));
                        } else {
                            clinit_code.push(Instruction::Sipush(sz));
                        }
                        clinit_code.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int));
                        clinit_code.push(Instruction::Putstatic(fr));
                    }
                } else if self.global.global_struct_offset_sets.contains_key(gname) {
                    let offsets = self.global.global_struct_offset_sets.get(gname).unwrap();
                    let size = (offsets.iter().max().unwrap_or(&0) / 4 + 1) as i16;
                    if size <= 127 {
                        clinit_code.push(Instruction::Bipush(size as i8));
                    } else {
                        clinit_code.push(Instruction::Sipush(size));
                    }
                    clinit_code.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int));
                    clinit_code.push(Instruction::Putstatic(fr));
                }
            }
        }
        clinit_code.push(Instruction::Return);
        let clinit_max = 3;
        methods.push(Method {
            access_flags: MethodAccessFlags::STATIC,
            name_index: clinit_name,
            descriptor_index: clinit_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: clinit_max, max_locals: 0,
                code: clinit_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let mut fields = vec![];
        if count > 0 {
            fields.push(Field {
                access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
                name_index: coro_field_name,
                descriptor_index: coro_field_desc,
                field_type: FieldType::parse("[Ljava/lang/Object;")
                    .expect("Failed to parse coro field type"),
                attributes: vec![],
            });
        }
        fields.push(Field {
            access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
            name_index: file_fds_name,
            descriptor_index: file_fds_desc,
            field_type: FieldType::parse("[Ljava/io/InputStream;")
                .expect("Failed to parse file fds field type"),
            attributes: vec![],
        });
        fields.push(Field {
            access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
            name_index: file_next_name,
            descriptor_index: file_next_desc,
            field_type: FieldType::parse("I")
                .expect("Failed to parse file next field type"),
            attributes: vec![],
        });

        // === Global variable fields (from user code) ===
        for (gname, gty) in &self.global.global_vars {
            let desc = self.global_jvm_descriptor(gname, gty);
            let name_idx = self.pool.constant_pool.add_utf8(gname).expect("utf8");
            let desc_idx = self.pool.constant_pool.add_utf8(&desc).expect("utf8");
            fields.push(Field {
                access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
                name_index: name_idx,
                descriptor_index: desc_idx,
                field_type: FieldType::parse(&desc)
                    .expect("Failed to parse global var field type"),
                attributes: vec![],
            });
        }

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5,
            constant_pool: self.pool.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces: vec![],
            fields,
            methods,
            attributes: vec![],
            code_source_url: None,
        };
        let mut buf = Vec::new();
        match class_file.to_bytes(&mut buf) {
            Ok(()) => buf,
            Err(e) => panic!("RuntimeStub serialization: {e:?}"),
        }
    }

    fn build_coro_init(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        count: usize,
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let init_name = self.pool.constant_pool.add_utf8("coro_init")
            .expect("Failed to add 'coro_init' UTF8");
        let init_desc = self.pool.constant_pool.add_utf8("()V")
            .expect("Failed to add coro_init descriptor");
        let mut c = Vec::new();
        push_iconst(&mut c, count);
        c.push(Instruction::Anewarray(
            self.pool.constant_pool.add_class("java/lang/Object")
                .expect("Failed to add Object class")
        ));
        c.push(Instruction::Putstatic(coro_field_ref));
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            let ir = self.pool.constant_pool.add_method_ref(*ci, "<init>", "()V")
                .expect("Failed to add init ref");
            c.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut c, i);
            c.push(Instruction::New(*ci));
            c.push(Instruction::Dup);
            c.push(Instruction::Invokespecial(ir));
            c.push(Instruction::Aastore);
        }
        c.push(Instruction::Return);
        let max_init_stack = COROUTINE_MAX_STACK.max(2 + count as u16);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: init_name,
            descriptor_index: init_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: max_init_stack,
                max_locals: 0,
                code: c,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_coro_resume(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let resume_name = self.pool.constant_pool.add_utf8("resume_coroutine")
            .expect("Failed to add 'resume_coroutine' UTF8");
        let resume_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("Failed to add resume descriptor");
        let mut code = Vec::new();
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            code.push(Instruction::Iload_0);
            push_iconst(&mut code, i);
            let skip_at = code.len();
            code.push(Instruction::If_icmpne(0));
            code.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut code, i);
            code.push(Instruction::Aaload);
            code.push(Instruction::Checkcast(*ci));
            let rm = self.pool.constant_pool.add_method_ref(*ci, "resume", "()I")
                .expect("Failed to add resume method ref");
            code.push(Instruction::Invokevirtual(rm));
            code.push(Instruction::Ireturn);
            code[skip_at] = Instruction::If_icmpne(code.len() as u16);
        }
        code.push(Instruction::Iconst_1);
        code.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: resume_name,
            descriptor_index: resume_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: COROUTINE_MAX_STACK,
                max_locals: 1,
                code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_coro_get_state(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let state_name = self.pool.constant_pool.add_utf8("get_coroutine_state")
            .expect("Failed to add 'get_coroutine_state' UTF8");
        let state_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("Failed to add get_state descriptor");
        let mut sc = Vec::new();
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            sc.push(Instruction::Iload_0);
            push_iconst(&mut sc, i);
            let skip_at = sc.len();
            sc.push(Instruction::If_icmpne(0));
            sc.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut sc, i);
            sc.push(Instruction::Aaload);
            sc.push(Instruction::Checkcast(*ci));
            let sm = self.pool.constant_pool.add_method_ref(*ci, "getState", "()I")
                .expect("Failed to add getState method ref");
            sc.push(Instruction::Invokevirtual(sm));
            sc.push(Instruction::Ireturn);
            sc[skip_at] = Instruction::If_icmpne(sc.len() as u16);
        }
        sc.push(Instruction::Iconst_m1);
        sc.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: state_name,
            descriptor_index: state_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: COROUTINE_MAX_STACK,
                max_locals: 1,
                code: sc,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_coro_set_param(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let set_name = self.pool.constant_pool.add_utf8("set_coroutine_param")
            .expect("Failed to add 'set_coroutine_param' UTF8");
        let set_desc = self.pool.constant_pool.add_utf8("(III)V")
            .expect("Failed to add set_param descriptor");
        let mut set_code = Vec::new();
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            set_code.push(Instruction::Iload_0);
            push_iconst(&mut set_code, i);
            let skip_at = set_code.len();
            set_code.push(Instruction::If_icmpne(0));
            set_code.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut set_code, i);
            set_code.push(Instruction::Aaload);
            set_code.push(Instruction::Checkcast(*ci));
            let (p1_ref, p2_ref) = self.coro.coroutine_param_field_refs[i];
            if let Some(fr) = p1_ref {
                set_code.push(Instruction::Dup);
                set_code.push(Instruction::Iload_1);
                set_code.push(Instruction::Putfield(fr));
            }
            if let Some(fr) = p2_ref {
                set_code.push(Instruction::Dup);
                set_code.push(Instruction::Iload_2);
                set_code.push(Instruction::Putfield(fr));
            }
            set_code.push(Instruction::Pop);
            set_code.push(Instruction::Return);
            set_code[skip_at] = Instruction::If_icmpne(set_code.len() as u16);
        }
        set_code.push(Instruction::Return);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: set_name,
            descriptor_index: set_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: COROUTINE_MAX_STACK,
                max_locals: COROUTINE_MAX_LOCALS,
                code: set_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }
}

fn push_iconst(c: &mut Vec<Instruction>, n: usize) {
    match n {
        0 => c.push(Instruction::Iconst_0),
        1 => c.push(Instruction::Iconst_1),
        2 => c.push(Instruction::Iconst_2),
        3 => c.push(Instruction::Iconst_3),
        4 => c.push(Instruction::Iconst_4),
        5 => c.push(Instruction::Iconst_5),
        _ => c.push(Instruction::Bipush(n as i8)),
    }
}

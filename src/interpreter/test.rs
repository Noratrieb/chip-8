use crate::interpreter::{Chip8Vm, execute};

#[test]
fn instructions_0_1_2() {
    let mut vm = Chip8Vm::default();
    //frame clearing is not implemented yet.
    vm.pc = 10;
    execute(0x20FF, &mut vm); // call 0FF
    assert_eq!(vm.pc + 1, 0x0FF);
    assert_eq!(vm.sp, 1);
    vm.pc = 0x0FF;
    execute(0x00EE, &mut vm); // return
    assert_eq!(vm.sp, 0);
    assert_eq!(vm.pc, 10);
    execute(0x10AB, &mut vm); // jmp to 0AB
    assert_eq!(vm.pc + 1, 0x0AB);
}

#[test]
fn instructions_3_4_5() {
    let mut vm = Chip8Vm::default();
    vm.reg[0] = 0xF2;
    execute(0x30F2, &mut vm);
    assert_eq!(vm.pc, 1);
    execute(0x40F3, &mut vm);
    assert_eq!(vm.pc, 2);
    execute(0x40F2, &mut vm);
    assert_eq!(vm.pc, 2);
    vm.reg[1] = 0xF2;
    execute(0x5010, &mut vm);
    assert_eq!(vm.pc, 3);
}

#[test]
fn instructions_6_7() {
    let mut vm = Chip8Vm::default();
    execute(0x60AB, &mut vm);
    assert_eq!(vm.reg[0], 0xAB);
    execute(0x69FF, &mut vm);
    assert_eq!(vm.reg[9], 0xFF);
    execute(0x7001, &mut vm);
    assert_eq!(vm.reg[0], 0xAC);
}

/// macros to simplify testing the 8xyz instructions
macro_rules! test_reg_ops {
    ($i:literal: $x:literal $y:literal => $z:literal for $vm:ident) => {
        $vm.reg[0] = $x;
        $vm.reg[1] = $y;
        crate::interpreter::execute($i, &mut $vm);
        assert_eq!($vm.reg[0], $z);
    }
}
/// macros to simplify testing the 8xyz instructions
macro_rules! test_reg_ops_flag {
    ($i:literal: $x:literal $y:literal => $z:literal + $f:literal for $vm:ident) => {
        $vm.reg[0] = $x;
        $vm.reg[1] = $y;
        crate::interpreter::execute($i, &mut $vm);
        assert_eq!($vm.reg[0], $z);
        assert_eq!($vm.reg[0xF], $f);
    }
}

#[test]
fn instructions_8xyz() {
    let mut vm = Chip8Vm::default();
    vm.reg[1] = 2;
    // 0 store V1 in V0
    execute(0x8010, &mut vm);
    assert_eq!(vm.reg[0], 2);

    // 1 OR V0 and V1, expected result in V2
    test_reg_ops!(0x8011: 0b10011111 0b11010001 => 0b11011111 for vm);

    // 2 AND V0 and V1, expected result in V2
    test_reg_ops!(0x8012: 0b11011111 0b10010001 => 0b10010001 for vm);

    // 3 XOR V0 and V1, expected result in V2
    test_reg_ops!(0x8013: 0b11011111 0b10010001 => 0b01001110 for vm);

    // 4 ADD V0 to V1, overflow => flag
    test_reg_ops_flag!(0x8014: 10 10 => 20 + 0 for vm);
    test_reg_ops_flag!(0x8014: 255 11 => 10 + 1 for vm);

    // 5 SUB V1 from V0, V0 > V1 => flag
    test_reg_ops_flag!(0x8015: 20 10 => 10 + 1 for vm);
    test_reg_ops_flag!(0x8015: 0 1 => 255 + 0 for vm);

    // 6 SHR Shift V0 right by one bit, overflow => flag
    test_reg_ops_flag!(0x8016: 0 0b10000000 => 0b01000000 + 0 for vm);
    test_reg_ops_flag!(0x8016: 0 0b00000001 => 0b00000000 + 1 for vm);

    // 7 Subtract V0 from V1, V1 > V0 => flag
    test_reg_ops_flag!(0x8017: 10 20 => 10 + 1 for vm);
    test_reg_ops_flag!(0x8017: 1 0 => 255 + 0 for vm);

    // E SHL Shift V0 left by one bit, overflow => flag
    test_reg_ops_flag!(0x801E: 0 0b00000001 => 0b00000010 + 0 for vm);
    test_reg_ops_flag!(0x801E: 0 0b10000000 => 0b00000000 + 1 for vm);
}

#[test]
fn instruction_9() {
    let mut vm = Chip8Vm::default();
    vm.reg[0] = 10;
    vm.reg[1] = 11;
    execute(0x9010, &mut vm);
    assert_eq!(vm.pc, 1);
    execute(0x9EE0, &mut vm);
    assert_eq!(vm.pc, 1);
}

#[test]
fn instruction_a_b_c() {
    let mut vm = Chip8Vm::default();
    execute(0xA123, &mut vm);
    assert_eq!(vm.i, 0x123);
    vm.reg[0] = 100;
    execute(0xB123, &mut vm);
    assert_eq!(vm.pc, 100 + 0x123 - 1); // - 1 because the emulator does the next() at the end normally

    execute(0xC000, &mut vm);
    assert_eq!(vm.reg[0], 0); // is 0 because the kk value is AND with the random value
    execute(0xC00F, &mut vm);
    assert!(vm.reg[0] < 0x10);
}

#[test]
fn instruction_f() {
    let mut vm = Chip8Vm::default();

    vm.delay_t = 245;
    execute(0xF007, &mut vm);
    assert_eq!(vm.reg[0], 245);

    execute(0xF115, &mut vm); // set delay_t = V1
    assert_eq!(vm.delay_t, 0);

    vm.reg[0] = 13;
    execute(0xF018, &mut vm); // set sound_t = V0
    assert_eq!(vm.sound_t, 13);

    vm.reg[0] = 1;
    vm.i = 10;
    execute(0xF01E, &mut vm); // i = i + v0
    assert_eq!(vm.i, 11);

    // Fx19 unimplemented


    vm.i = 0;
    vm.reg[0] = 255;
    vm.reg[1] = 99;
    vm.reg[2] = 1;
    execute(0xF033, &mut vm);
    vm.i = 3;
    execute(0xF133, &mut vm);
    vm.i = 6;
    execute(0xF233, &mut vm);

    assert_eq!(&vm.memory[0..9], &[2, 5, 5, 0, 9, 9, 0, 0, 1]);

    // store all registers from 0 including A in memory at i
    for i in 0..=0xAu8 {
        vm.reg[i as usize] = i + 1;
    }
    vm.i = 100;
    execute(0xFA55, &mut vm);
    assert_eq!(&vm.memory[100..=110], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    // load all registers from 0 including A from memory at i
    for i in 0..=0xAu8 {
        vm.memory[i as usize] = i + 1;
    }
    vm.i = 0;
    execute(0xFA65, &mut vm);
    assert_eq!(&vm.reg[0..=10], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}
use crate::interpreter::{Emulator, execute};

#[test]
fn instructions_0_1_2() {
    let mut em = Emulator::default();
    em.pc = 10;
    execute(0x20FF, &mut em); // call 0FF
    assert_eq!(em.pc + 1, 0x0FF);
    assert_eq!(em.sp, 1);
    em.pc = 0x0FF;
    execute(0x00EE, &mut em); // return
    assert_eq!(em.sp, 0);
    assert_eq!(em.pc, 10);
    execute(0x10AB, &mut em); // jmp to 0AB
    assert_eq!(em.pc + 1, 0x0AB);
}

#[test]
fn instructions_3_4_5() {
    let mut em = Emulator::default();
    em.reg[0] = 0xF2;
    execute(0x30F2, &mut em);
    assert_eq!(em.pc, 1);
    execute(0x40F3, &mut em);
    assert_eq!(em.pc, 2);
    execute(0x40F2, &mut em);
    assert_eq!(em.pc, 2);
    em.reg[1] = 0xF2;
    execute(0x5010, &mut em);
    assert_eq!(em.pc, 3);
}

#[test]
fn instructions_6_7() {
    let mut em = Emulator::default();
    execute(0x60AB, &mut em);
    assert_eq!(em.reg[0], 0xAB);
    execute(0x69FF, &mut em);
    assert_eq!(em.reg[9], 0xFF);
    execute(0x7001, &mut em);
    assert_eq!(em.reg[0], 0xAC);
}

#[test]
fn instruction_fx33_conversion() {
    let mut em = Emulator::default();
    em.reg[0] = 255;
    em.reg[1] = 99;
    em.reg[2] = 1;
    let instruction1 = 0xF033;
    let instruction2 = 0xF133;
    let instruction3 = 0xF233;
    execute(instruction1, &mut em);
    em.i = 3;
    execute(instruction2, &mut em);
    em.i = 6;
    execute(instruction3, &mut em);

    assert_eq!(&em.memory[0..9], &[2, 5, 5, 0, 9, 9, 0, 0, 1]);
}

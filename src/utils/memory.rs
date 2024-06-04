#[allow(clippy::missing_safety_doc)]

/*
 * Example i686 instruction:
 * 00b6165e  e8 5d fd  CALL  function_1
 *           ff ff
 *
 * To get address of function_1 you need:
 * 1. Get address of 00b6165e on runtime,
 * 2. Skip E8 opcode (call instruction)
 * 3. Pass address of 00b6165e + opcode offset as relative_address_pos
 *
 */
pub unsafe fn relative_to_absolute_i32(relative_address_pos: usize) -> usize {
    let relative_address_value = (relative_address_pos as *const i32).read_unaligned();

    let next_instruction = relative_address_pos + std::mem::size_of::<i32>();

    match super::from_i32::FromI32::new(relative_address_value) {
        super::from_i32::FromI32::Positive(value) => next_instruction + value as usize,
        super::from_i32::FromI32::Negative(value) => next_instruction - value as usize,
    }
}

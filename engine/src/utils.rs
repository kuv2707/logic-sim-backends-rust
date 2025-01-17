pub fn form_expr(inex: &Vec<String>, sym: &String) -> String {
    // some operators would be infix, some prefix.
    if inex.len() == 1 {
        return format!("{}{}", sym, inex[0]);
    }
    return format!("{}{}{}", inex[0], sym, inex[1]);
}

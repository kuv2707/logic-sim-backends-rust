pub fn form_expr(inex: &Vec<String>, sym: &String) -> String {
    // some operators would be infix, some prefix.
    if inex.len() == 2 {
        return format!("{}{}", sym, inex[1]);
    }
    //todo: extend for more
    return format!("({})", inex[1..].join(sym));
    // return format!("{}{}{}", inex[1], sym, inex[2]);
}

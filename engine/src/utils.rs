pub fn form_expr(inex: &Vec<String>, sym: &String) -> String {
    // some operators would be infix, some prefix.
    return format!("{}({})", sym, inex.join(","));
}

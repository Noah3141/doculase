pub fn validate_env_vars() {
    let environment_variables = [
        "DATABASE_URL",
        "CHATGPT_API_KEY"
    ];

    for var in environment_variables {
        dotenvy::var(var).expect(format!("\n\n{var} variable not found!\n\n").as_str());
    }
}
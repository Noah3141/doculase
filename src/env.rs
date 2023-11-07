use std::collections::HashMap;

const ENV_VARS: [&str; 2]= [
    "DATABASE_URL",
    "CHATGPT_API_KEY"
];


pub fn validate_env_vars() -> HashMap<&'static str, String> {

    let mut env_vars: HashMap<&str, String> = HashMap::new();

    for var in ENV_VARS {
        env_vars.insert(
            var, 
            dotenvy::var(var).expect(
                format!("\n\n{var} variable not found!\n\n").as_str()
            )
        );
    }

    env_vars
}
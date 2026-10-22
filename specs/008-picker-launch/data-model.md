# Data model: Picker launch

`LaunchPlan = Found{ source: Config|Env|Xdg|List, argv: Vec<String> } | NotFound{ searched: Vec<String> }`; `TERMINALS: &[(name, &[prefix_args])]`.

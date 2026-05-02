/// Maximum session name length to avoid zellij IPC socket path limit.
/// Zellij has a 103-byte limit for Unix domain socket paths.
/// On macOS, TMPDIR can be ~49 chars + zellij path ~31 chars = ~80 chars prefix.
/// 32 chars leaves headroom for the session name while staying under the limit.
const MAX_SESSION_NAME_LENGTH: usize = 32;

/// Sanitizes a workspace name into a tmux/zellij-safe session name.
/// Matches the exact vscode-mux algorithm with added length limit for zellij compatibility:
/// - Replace any char not in [a-zA-Z0-9-] with '-'
/// - Collapse consecutive '-' into one
/// - Strip leading/trailing '-'
/// - Truncate to MAX_SESSION_NAME_LENGTH to avoid zellij socket path limits
/// - Return 'session' if result is empty
#[inline]
pub fn sanitize_session_name(name: &str) -> String {
    // Step 1: Replace any character not in [a-zA-Z0-9-] with '-'
    let mut result: Vec<char> = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Step 2: Collapse consecutive '-' into single '-'
    let mut collapsed = Vec::with_capacity(result.len());
    let mut prev_was_dash = false;
    for c in result {
        if c == '-' {
            if !prev_was_dash {
                collapsed.push(c);
            }
            prev_was_dash = true;
        } else {
            collapsed.push(c);
            prev_was_dash = false;
        }
    }
    result = collapsed;

    // Step 3 & 4: Strip leading and trailing '-'
    let start = result
        .iter()
        .position(|&c| c != '-')
        .unwrap_or(result.len());
    if start > 0 {
        result.drain(0..start);
    }
    while result.last() == Some(&'-') {
        result.pop();
    }

    // Step 5: Return 'session' if result is empty
    if result.is_empty() {
        return "session".to_string();
    }

    // Step 6: Truncate to max length to avoid zellij IPC socket path limits
    let mut result: String = result.into_iter().collect();
    if result.len() > MAX_SESSION_NAME_LENGTH {
        // Truncate from the end to preserve the beginning of the name
        result.truncate(MAX_SESSION_NAME_LENGTH);
    }

    result
}

/// Computes a unique session name with gap-filling (matches vscode-mux exactly).
/// If base is not in sessions, returns base unchanged.
/// Otherwise starts at suffix=2 and finds the first available gap.
/// Example: sessions=['myapp','myapp-2','myapp-5'] → returns 'myapp-3'
#[inline]
pub fn get_unique_session_name(base: &str, sessions: &[String]) -> String {
    // Linear search is faster than HashSet for small lists (typical case < 10 sessions)
    let contains = |s: &str| sessions.iter().any(|session| session == s);

    if !contains(base) {
        return base.to_string();
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

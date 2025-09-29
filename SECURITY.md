# Security Guidelines

## Shell Command Execution Security

Rainy CLI includes robust security measures for shell command execution to protect your system while enabling powerful AI-driven automation.

### Security Levels

The shell execution system operates with three configurable security levels:

#### Low Security (Most Restrictive)

- **Always requires user approval** for any command execution
- Suitable for production environments or when maximum control is needed
- All commands are validated and presented to the user before execution

#### Medium Security (Default)

- **Requires approval for risky operations** only
- Safe commands (like `ls`, `echo`, `git status`) execute automatically
- Risky commands (file system modifications, network operations) require approval
- Recommended for most development environments

#### High Security (Least Restrictive)

- **Executes all validated commands automatically**
- Still blocks dangerous patterns and commands
- Suitable for trusted environments and experienced users
- Fastest execution with minimal interruptions

### Command Validation

All commands go through multiple validation layers:

#### 1. Command Categorization

Commands are automatically categorized by risk level:

- **Safe**: `ls`, `echo`, `pwd`, `git status`, `cat` (read-only operations)
- **FileSystem**: `cp`, `mv`, `mkdir`, `touch` (file operations)
- **PackageManagement**: `npm`, `pip`, `cargo`, `apt` (package managers)
- **Network**: `curl`, `wget`, `ping` (network operations)
- **SystemAdmin**: `ps`, `top`, `systemctl` (system administration)
- **Dangerous**: Unknown commands or those with high risk potential

#### 2. Dangerous Pattern Detection

The system automatically blocks commands containing dangerous patterns:

- **Destructive operations**: `rm -rf`, `del /s`, `format`
- **Privilege escalation**: `sudo rm`, `su -c`
- **System modification**: `chmod 777`, `chown root`
- **Remote execution**: `curl | sh`, `wget | bash`
- **Process termination**: `killall`, `pkill -9`

#### 3. Allow/Block Lists

- **Allowed commands**: Explicitly permitted commands that bypass some restrictions
- **Blocked commands**: Commands that are never allowed to execute
- **Configurable**: Can be customized per project or user preferences

### Configuration

Shell security can be configured through the `ShellConfig`:

```rust
use rainy_cli::shell::{ShellConfig, SecurityLevel};

let config = ShellConfig {
    security_level: SecurityLevel::Medium,
    timeout_seconds: 300,
    allowed_commands: vec!["git".to_string(), "cargo".to_string()],
    blocked_commands: vec!["rm".to_string(), "del".to_string()],
    working_directory: Some(PathBuf::from("/safe/directory")),
    environment_vars: HashMap::new(),
};
```

### Best Practices

#### For Users

1. **Start with Medium security** - provides good balance of safety and usability
2. **Review approval prompts carefully** - understand what commands will be executed
3. **Use project-specific configurations** - different projects may need different security levels
4. **Monitor command execution logs** - keep track of what the AI is doing
5. **Report suspicious behavior** - if the AI attempts dangerous operations

#### For Developers

1. **Always validate user input** before passing to shell execution
2. **Use the lowest security level appropriate** for your use case
3. **Implement proper error handling** for command failures
4. **Log all command executions** for audit purposes
5. **Test with different security levels** during development

### Environment Isolation

The shell executor provides several isolation mechanisms:

#### Working Directory Restriction

- Commands execute in a specified working directory
- Prevents accidental operations outside project scope
- Can be configured per execution context

#### Environment Variable Control

- Custom environment variables can be set
- Sensitive variables can be filtered out
- Prevents leakage of credentials or secrets

#### Timeout Protection

- All commands have configurable timeouts
- Prevents runaway processes from consuming resources
- Default timeout is 5 minutes, adjustable per command

### Audit and Monitoring

#### Command Logging

All executed commands are logged with:

- Command text and arguments
- Execution timestamp
- Exit code and duration
- Security level used
- User approval status

#### Error Reporting

Security violations are reported with:

- Reason for blocking
- Suggested alternatives
- Security level recommendations
- Pattern matches that triggered the block

### Emergency Procedures

#### If a Dangerous Command is Executed

1. **Stop the process immediately** if still running
2. **Check system integrity** - verify no damage occurred
3. **Review the command log** - understand how it happened
4. **Update security configuration** - prevent similar incidents
5. **Report the issue** - help improve the security system

#### If the AI Behaves Suspiciously

1. **Switch to Low security level** immediately
2. **Review recent command history**
3. **Check for unusual patterns** in AI requests
4. **Consider resetting the AI context**
5. **Report the behavior** for investigation

### Security Updates

The security system is continuously updated with:

- New dangerous pattern detection
- Improved command categorization
- Enhanced validation rules
- Community-reported security issues

Keep your Rainy CLI installation updated to receive the latest security improvements.

## Reporting Security Issues

If you discover a security vulnerability, please report it to:

- **Email**: <security@rainy-cli.dev>
- **GitHub**: Create a private security advisory
- **Priority**: Critical security issues will be addressed within 24 hours

Please include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested mitigation if known

## Security Acknowledgments

We thank the security research community for helping keep Rainy CLI secure. Responsible disclosure is appreciated and will be acknowledged in our security advisories.

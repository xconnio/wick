# wcli - WAMP Command Line Interface

A command-line tool for interacting with WAMP (Web Application Messaging Protocol) routers. Supports calling procedures, publishing events, registering procedures, and subscribing to topics.

## Installation

```bash
cargo build --release
```

## Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--url <URL>` | Router URL to connect to | `ws://localhost:8080/ws` |
| `--realm <REALM>` | Realm to join | `realm1` |
| `--serializer <TYPE>` | Serializer (json, msgpack, cbor) | `json` |
| `--authid <ID>` | Authentication ID | - |
| `--authrole <ROLE>` | Authentication role | - |
| `--ticket <TICKET>` | Ticket for ticket authentication | - |
| `--secret <SECRET>` | Secret for Challenge-Response Auth | - |
| `--private-key <HEX>` | ed25519 private key for cryptosign | - |

---

## Commands

### Call Procedure

Call a remote procedure with arguments.

```bash
# Basic call
wcli call com.example.add 1 2

# Call with keyword arguments
wcli call com.example.greet -k name=Alice -k age=30

# Call with options
wcli call com.example.fetch -o timeout=5000

# Stress testing: call 100 times across 10 parallel sessions
wcli call com.example.ping --repeat 100 --parallel 10 --concurrency 5
```

**Options:**
- `-k, --kwarg KEY=VALUE` - Keyword argument (repeatable)
- `-o, --option KEY=VALUE` - WAMP call option (repeatable)
- `--repeat N` - Number of calls per session (default: 1)
- `--parallel N` - Number of parallel sessions (default: 1)
- `--concurrency N` - Maximum concurrent sessions (default: 1)

---

### Register Procedure

Register a procedure and handle incoming invocations.

```bash
# Register a procedure
wcli register com.example.echo

# The handler prints received args/kwargs as JSON and echoes them back
# Press Ctrl+C to exit
```

---

### Subscribe to Topic

Subscribe to a topic and receive events.

```bash
# Subscribe to a topic
wcli subscribe com.example.events

# Multiple parallel subscribers
wcli subscribe com.example.events --parallel 10

# With concurrency limit
wcli subscribe com.example.events --parallel 100 --concurrency 10
```

**Options:**
- `--parallel N` - Number of parallel sessions to subscribe (default: 1)
- `--concurrency N` - Maximum concurrent sessions (default: 1)

---

### Publish to Topic

Publish events to a topic.

```bash
# Basic publish
wcli publish com.example.events "Hello, World!"

# Publish with arguments
wcli publish com.example.events 42 true "message"

# Publish with keyword arguments
wcli publish com.example.events -k user=Alice -k action=login

# Request acknowledgement from the broker
wcli publish com.example.events "data" --acknowledge

# Stress testing: publish 1000 events
wcli publish com.example.events "test" --repeat 1000 --parallel 10
```

**Options:**
- `-k, --kwarg KEY=VALUE` - Keyword argument (repeatable)
- `-o, --option KEY=VALUE` - WAMP publish option (repeatable)
- `--acknowledge` - Request acknowledgement from broker
- `--repeat N` - Number of publishes per session (default: 1)
- `--parallel N` - Number of parallel sessions (default: 1)
- `--concurrency N` - Maximum concurrent sessions (default: 1)

---

### Generate Keypair

Generate an ed25519 keypair for cryptosign authentication.

```bash
# Generate and display keypair
wcli keygen

# Output:
# Public Key: 74bcb0f9636780f5a5d7bf319f5441b654203aad17aef92e01f70d161381da9d
# Private Key: ae742323f8da546407cc0abef409c714ba7e747e534e82632670dbf938a2b755

# Save keypair to files (key and key.pub)
wcli keygen -O

# Save with custom filename (my_keys and my_keys.pub)
wcli keygen -O my_keys
```

---

## Authentication

wcli supports multiple authentication methods. The priority order is:
1. **Cryptosign** (if `--private-key` is provided)
2. **WAMP-CRA** (if `--secret` is provided)
3. **Ticket** (if `--ticket` is provided)
4. **Anonymous** (default)

### Ticket Authentication

```bash
wcli --authid alice --ticket my-secret-ticket call com.example.protected
```

### WAMP-CRA (Challenge-Response Authentication)

```bash
wcli --authid alice --secret my-secret-password call com.example.protected
```

### Cryptosign Authentication

```bash
# First, generate a keypair
wcli keygen -O

# Use the private key for authentication
wcli --authid alice --private-key ae742323f8da546407cc0abef409c714ba7e747e534e82632670dbf938a2b755 call com.example.protected

# Or read from file
wcli --authid alice --private-key "$(cat key)" call com.example.protected
```

### With Authrole

```bash
wcli --authid alice --authrole admin --ticket my-ticket call com.example.admin_only
```

---

## Serializers

wcli supports three serialization formats:

### JSON (Default)

```bash
wcli --serializer json call com.example.echo "hello"
```

### MessagePack

More efficient binary serialization.

```bash
wcli --serializer msgpack call com.example.echo "hello"
```

### CBOR

Concise Binary Object Representation.

```bash
wcli --serializer cbor call com.example.echo "hello"
```

---

## Argument Types

Arguments are automatically parsed as:
- **Integer**: `42`, `-10`
- **Float**: `3.14`, `-0.5`
- **Boolean**: `true`, `false`
- **String**: `hello`, `"hello world"`

To force a value to be a string (e.g., the number `123` as a string), use quotes:

```bash
wcli call com.example.echo "'123'"
wcli call com.example.echo '"true"'
```

---

## Examples

### Complete Workflow

```bash
# Terminal 1: Register a procedure
wcli register com.example.add

# Terminal 2: Call the procedure
wcli call com.example.add 5 3

# Terminal 1: Subscribe to events
wcli subscribe com.example.notifications

# Terminal 2: Publish an event
wcli publish com.example.notifications "New message!" -k from=system
```

### Custom Router

```bash
wcli --url ws://router.example.com:8080/ws --realm production call com.api.status
```

### Stress Testing

```bash
# 10 parallel sessions, each making 100 calls, max 5 concurrent
wcli call com.example.ping --parallel 10 --repeat 100 --concurrency 5
```

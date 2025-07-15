# Rig Multi Agent

An example project demonstrating how to use the Rig framework and SurrealDB for building AI multi-agent executions.

This example uses two data sets, a customer table and a feature request table. It executes a prompt to prioritize feature requests. The Map agent splits the question into two sub questions, one for fetching customers by ARR, and another for fetching feature requests by urgency. The Reduce agent combines the results from the Map agent and provides a prioritized list of feature requests.

## Setup

### Prerequisites

- Rust (latest stable version)
- Access to xAI API
- SurrealDB instance (cloud or local)

### Environment Configuration

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit the `.env` file with your actual configuration values:

   ```env
   # xAI API Configuration
   XAI_API_KEY=your_xai_api_key_here

   # SurrealDB Configuration
   SURREAL_HOST=your_surreal_host_here
   SURREAL_USERNAME=your_username_here
   SURREAL_PASSWORD=your_password_here
   SURREAL_NAMESPACE=your_namespace_here
   SURREAL_DATABASE=your_database_here
   ```

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `XAI_API_KEY` | Your xAI API key | `xai-abc123...` |
| `SURREAL_HOST` | SurrealDB host URL | `your-instance.surreal.cloud` |
| `SURREAL_USERNAME` | SurrealDB username | `your_username` |
| `SURREAL_PASSWORD` | SurrealDB password | `your_password` |
| `SURREAL_NAMESPACE` | SurrealDB namespace | `your_namespace` |
| `SURREAL_DATABASE` | SurrealDB database name | `your_database` |

## Running the Application

1. Install dependencies:
   ```bash
   cargo build
   ```

2. Run the application:
   ```bash
   cargo run
   ```

## Project Structure

- `src/main.rs` - Main application entry point
- `src/config.rs` - Environment configuration management
- `src/agents/` - AI agent implementations
- `src/surreal/` - SurrealDB integration tools

## Features

This demonstrates:
- Loading configuration from environment variables
- Creating AI agents with the Rig framework
- Integrating with SurrealDB for data queries
- Multi-agent workflows for complex question answering
- Feature request prioritization based on customer value

## Error Handling

The application includes comprehensive error handling for configuration issues:

- **Missing Environment Variables**: The app will clearly indicate which environment variables are missing
- **Invalid API Key Format**: xAI API keys must start with "xai-"
- **Empty Configuration Values**: Host, username, and password cannot be empty
- **Clear Error Messages**: All configuration errors include helpful instructions

### Common Error Messages

```
Error loading configuration: Missing environment variable: XAI_API_KEY
Please check your .env file or environment variables.
Copy .env.example to .env and fill in your values.
```

```
Error loading configuration: Invalid configuration value: XAI_API_KEY must start with 'xai-'
```

## Troubleshooting

2. **Database Connection Issues**:
   - Verify your SurrealDB host is accessible
   - Check that your username and password are correct
   - Ensure the namespace and database exist

3. **API Issues**:
   - Confirm your xAI API key is valid and has proper permissions
   - Check your internet connection for API calls

## Security

- Never commit your `.env` file to version control
- Keep your API keys secure and rotate them regularly
- Use environment-specific configurations for different deployments

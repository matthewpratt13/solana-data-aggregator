# Solana Data Aggregator

This application monitors a specified Solana blockchain address for new transactions, processes them and stores the relevant transaction data in a PostgreSQL database. It also provides a REST API to retrieve stored transaction data.

## Features

- Monitors a Solana address for new transactions using the Solana RPC client
- Extracts and validates transaction details (sender, receiver, SOL amount, fees, etc.)
- Stores transaction data in a PostgreSQL database
- Provides an API to query stored transactions

## Getting Started

### Prerequisites

To run this application, you need to have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL](https://www.postgresql.org/download/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (optional for interacting with the Solana network)
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) for database migrations.

### Environment Setup

1. Create a `.env` file in the root of your project with the following environment variables:

   ```bash
   DATABASE_URL=postgres://your_user:your_password@localhost/your_db
   RPC_URL=https://api.devnet.solana.com  # or another Solana RPC endpoint
   ADDRESS_A=YourSolanaAddressHere  # The public key of the address you want to monitor
   ```

   Replace `your_user`, `your_password`, and `your_db` with your PostgreSQL credentials, and replace `YourSolanaAddressHere` with the Solana public key you want to monitor.

2. Install the `sqlx-cli` tool to manage database migrations:

   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

3. Set up the PostgreSQL database using `sqlx-cli`:

   ```bash
   sqlx database setup
   ```

   This command will create the database and run all migrations.

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/your-username/solana-data-aggregator.git
   cd solana-data-aggregator
   ```

2. Install dependencies:

   ```bash
   cargo build
   ```

3. Prepare your database:

   - If the database has not been initialized yet, you can use `sqlx-cli` to run migrations.

     Create the migration files by running:

     ```bash
     sqlx migrate add create_transactions_table
     ```

     Then, add the SQL to create the `transactions` table:

     ```sql
     CREATE TABLE IF NOT EXISTS transactions (
         id SERIAL PRIMARY KEY,
         signature VARCHAR NOT NULL,
         sender VARCHAR NOT NULL,
         receiver VARCHAR NOT NULL,
         sol_amount BIGINT NOT NULL,
         fee BIGINT NOT NULL,
         timestamp BIGINT NOT NULL,
         prev_blockhash VARCHAR NOT NULL
     );
     ```

     Apply the migration using:

     ```bash
     sqlx migrate run
     ```

4. Verify that the migrations were successful by checking if the `transactions` table exists in your PostgreSQL database.

### Running the Application

1. Run the application with the following command:

   ```bash
   cargo run
   ```

2. The application will:
   - Monitor the specified Solana address for new transactions.
   - Process and store valid transactions in the PostgreSQL database.
   - Start a REST API server on `http://127.0.0.1:8080`.

### REST API

The API exposes the following endpoint:

- **GET** `/transactions` - Retrieve all stored transactions.

Example request:

```bash
curl http://127.0.0.1:8080/transactions
```

Example response:

```json
[
  {
    "signature": "5Fv6v3F56AkPp5Kiw67syT7oDfp4h5AdB8GHMZocQ5HEbkq",
    "sender": "5y5S1fgg1tNYBqJWueSh2HeckUhvLXruMWweZjsn7bEG",
    "receiver": "3RZPCdhvTz44bRJWCBszRoeZtE7Xr9uhEka7jKsqhyyE",
    "sol_amount": 5000000,
    "fee": 5000,
    "timestamp": 1638893200,
    "prev_blockhash": "5bQf4skisDdCE57sQvjqLRT9AtyfiSdLB2CUfuN14J5T"
  }
]
```

### Monitoring Solana Blockchain

The application continuously monitors the blockchain for transactions related to the specified address. It does this every 10 seconds (adjustable in the code) and stores valid transactions in the PostgreSQL database.

### Testing

To run the tests, use:

```bash
cargo test
```

Make sure the database is running before executing tests.

### Deployment

You can deploy the application using a service like [Heroku](https://www.heroku.com/), [DigitalOcean](https://www.digitalocean.com/), or any other cloud provider that supports Rust and PostgreSQL.

### Configuration

You can customize the behavior of the application by modifying the following:

- **Database Connection Pool:** Modify the connection pool configuration in `data_storage.rs`.
- **Transaction Validation:** Customize transaction validation logic in `data_processing.rs`.
- **Monitoring Interval:** Adjust the interval for monitoring the blockchain in `data_retrieval.rs` (`Duration::from_secs(10)`).

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request.

## License

This project is licensed under the GPL v3 License - see the [LICENSE](LICENSE) file for details.
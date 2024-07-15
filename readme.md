# Rocket-Auth

POC work for making authentication API including successfull signup/signin using Google Outh & email OTPs etc
Built in Rust(Rocket, diesel), Postgres.

## Prerequisites

Before running the application, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
  - After installing rust, run command `rustup install nightly-2018-10-05` to install older version
- `diesel-cli` (run `cargo install diesel_cli`)

## Run

1. **Clone the repository:**

   ```bash
   $ git clone https://github.com/HN026/rocket-auth.git
   $ cd rocket-auth

   ```

2. **Set up environment variables:**
   Create a .env file in the root directory of the project and add the necessary environment variables:

   ```bash
   DATABASE_URL=postgres://myuser:mysecretpassword@localhost/mydatabase
   ```

   Replace myuser, mysecretpassword, and mydatabase with the credentials and database name specified in your Docker Compose file.

3. **Run diesel migrations to setup database**

   ```bash
   $ diesel migration run
   ```

4. **Run the server**
   ```bash
   $ cargo +nightly run
   ```

## Usage

You will be successfully connected to `http://0.0.0.0:8000` address

- SignUp:

```bash
curl -X POST http://0.0.0.0:8000/signup -H "Content-Type: application/json" -d '{"username":"USER","email": "hello123@gmail.com", "password": "Hdjello"}'
```

- SignIn:

```bash
curl -X POST http://0.0.0.0:8000/signin -H "Content-Type: application/json" -d '{"username":"USER","email": "hello123@gmail.com", "password": "Hdjello"}'
```

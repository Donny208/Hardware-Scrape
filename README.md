# Hardware-scrape

**Hardware-scrape** is a Rust-built scraper that uses the Reddit API to find the best PC part deals based on user-provided keywords. It stores information in a PostgreSQL database and runs using Docker for easy deployment.

## Features
- Scrape Reddit for the latest PC part deals using user-defined keywords.
- Store and manage deal information in a PostgreSQL database.
- Simple deployment with Docker and Docker Compose.

## Requirements
- Rust (for building the scraper)
- PostgreSQL
- Docker & Docker Compose

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/hardware-scrape.git
   cd hardware-scrape

2. **Set up environment variables:**

   Create a `.env` file in the root of the project directory and set the following variables:
   ```
   REDDIT_API_KEY=your_reddit_api_key
   REDDIT_API_SECRET=your_reddit_api_secret
   POSTGRES_USER=your_postgres_user
   POSTGRES_PASSWORD=your_postgres_password
   POSTGRES_DB=hardware_scrape
   ```
   
3. Build and run with Docker Compose:
      
   `docker-compose up -d`

   This command will start the scraper and PostgreSQL database in detached mode.


## Usage

After running the scraper, it will monitor Reddit for posts matching the keywords provided by users.
Data is stored in the PostgreSQL database and can be queried for insights on PC part deals.
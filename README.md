
<img src="./.resources/logo.jpg" alt="Logo" width="250"/>

# RustyBalancer

RustyBalancer is a load balancer and deployment engine prototype, featuring:
- **Round Robin with Priorities** for distribution
- **Proactive Handling**
- **Load Balancer**
- **Deployment Engine**

## Installation Instructions

### Prerequisites

- **Docker** installed
- **Rust** installed

### Steps

1. **Clone the repository:**
   ```bash
   git clone https://github.com/mxmueller/RustyBalancer.git
   cd RustyBalancer
   ```
2. **Add current user to Docker group (if needed):**
   ```bash
   sudo usermod -aG docker $USER
   ```

3. **Build and run using Docker Compose:**
   ```bash
   docker-compose up --build
   ```
   
4. **Access the dashboard:**
   Open your web browser and go to `http://localhost:YOUR_PORT`.


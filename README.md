# Apollo 11 AGC Simulation Setup Guide

This guide provides step-by-step instructions to set up and run the **Apollo 11 AGC (Apollo Guidance Computer)** simulation project.  
The project includes a **Rust-based backend** and a **virtual DSKY (Display and Keyboard)** interface.

---

## Prerequisites

- **Operating System:** Linux/macOS/Windows (steps may vary for Windows and macOS)  
- **Hardware:** Minimum 4GB RAM (recommended) and sufficient storage for dependencies  
- **Accounts:** Git (required to clone repositories)

---

## Backend Setup

### 1. Install Rust and Cargo
Follow the official [Rust installation guide](https://www.rust-lang.org/tools/install).

---

### 2. Clone the Project Repository
Use Git to clone your AGC simulation project repository.

```bash
git clone <your-repository-url>
cd <your-repository>
````

---

### 3. Download and Build VirtualAGC

a. Clone the VirtualAGC repository:

```bash
git clone https://github.com/virtualagc/virtualagc.git
cd virtualagc
```

b. Install required dependencies for the VirtualAGC project.

c. Replace DSKY Files:

Copy your modified files into the correct directory:

```bash
cp path/to/yaDSKY2.cpp path/to/yaDSKY2.h virtualagc/yaDSKY2/
```

d. Build VirtualAGC:

```bash
make
```

---

### 4. Prepare Modified `yaDSKY2` and `oct2bin`

a. Copy the built `yaDSKY2` folder to your repository.

b. (Optional) Clone the VirtualAGC repository again if needed for additional tools.

---

### 5. Generate AGC Binaries

a. Convert `.binsource` files to binaries:

```bash
./oct2bin < Comanche055.binsource > Comanche055.bin
```

b. Move generated binaries into your project directory.

---

### 6. Build and Run the Rust AGC

a. Navigate to the Rust project folder:

```bash
cd path/to/rust/project
```

b. Run the AGC simulation (choose `luminary99` or `comanche55`):

```bash
cargo run -- luminary99
```

---

### 7. Connect yaDSKY

a. In a new terminal, navigate to the `yaDSKY2` folder:

```bash
cd virtualagc/yaDSKY2
```

b. Start the DSKY interface:

```bash
./yaDSKY
```

The DSKY interface should now connect to the AGC simulation.

---

## Output

The DSKY will output data to a file, saving all the **R3 register values** for reference.


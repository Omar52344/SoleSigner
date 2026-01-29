# SoleSigner UI

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Run the development server:
   ```bash
   npm run dev
   ```

## Configuration

- The API URL is set to `http://localhost:3000` in `lib/utils.ts`.
- Ensure the Rust backend is running on port 3000.

## Features

- **Vote**: Go to `/vote/[election_id]`.
- **Admin**: Go to `/admin/create`.
- **Audit**: Go to `/verify`.

## Tech Stack

- Next.js 14+ (App Router)
- React Query
- Tailwind CSS
- Zod, React Hook Form
- React Webcam

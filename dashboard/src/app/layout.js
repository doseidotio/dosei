'use client'

import "./globals.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";


export default function RootLayout({ children }) {
  const [queryClient] = useState(() => new QueryClient());
  return (
    <QueryClientProvider client={queryClient}>
      <html lang="en">
      <head>
        <title>Dosei</title>
        <link rel="icon" href="/logo-no-text-white.svg" sizes="any"/>
      </head>
      <body>{children}</body>
      </html>
    </QueryClientProvider>
  );
}

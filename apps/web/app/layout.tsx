import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "REACTION XI | Adaptive Football Decision Intelligence",
  description: "Compare player strengths, simulate opponent responses, and evaluate tactical corrections using synthetic data.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

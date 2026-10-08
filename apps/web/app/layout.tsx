import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "MatchLens AI | Explainable Football Intelligence",
  description: "Evidence-backed synthetic football match intelligence, powered by Rust.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

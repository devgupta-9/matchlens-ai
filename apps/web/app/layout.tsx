import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "ReactCoach 11 | AI-Powered Adaptive Football Coaching",
  description: "Assess player strengths and opponent vulnerabilities, compare tactical decisions and explore corrective coaching in synthetic football scenarios.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

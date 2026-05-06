import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Claw Code",
  description: "AI-powered coding assistant",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

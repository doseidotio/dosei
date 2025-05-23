'use client';

import Link from "next/link";
import { Container } from "@/components/Container";
import {HeroPattern} from "@/components/HeroPattern";
import {Footer} from "@/components/Footer";

export default function Page() {
  return (
    <>
      <div className="flex flex-col min-h-screen antialiased">
        <HeroPattern/>
        <main className="flex flex-col flex-1 bg-white/5">
          <nav className="p-6">
            <Link href="/" aria-label="Home">
              <img alt="Dosei Logo" src="/logo-no-text.svg" className="h-8 w-auto block dark:hidden"/>
              <img alt="Dosei Logo" src="/logo-no-text-white.svg" className="h-8 w-auto hidden dark:block"/>
            </Link>
          </nav>
          <Container className="flex flex-1 items-center">
            <div className="mx-auto text-center items-center flex flex-col justify-center space-y-4 py-16">
              <h1 className="text-3xl font-semibold mb-6">
                Dosei CLI Login Success
              </h1>
              <p>Dosei CLI was successfully authenticated.</p>
              <p>You can close now this tab and return to the Dosei CLI</p>
            </div>
          </Container>
        </main>
        <Footer/>
      </div>
    </>
  )
}

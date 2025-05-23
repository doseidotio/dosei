'use client';
import Link from "next/link";
import { Container } from "@/components/Container";
import {HeroPattern} from "@/components/HeroPattern";
import {Footer} from "@/components/Footer";
import {useRouter, useSearchParams} from "next/navigation";
import {useEffect, Suspense} from "react";
import {storeSessionFromString, getSession } from "@/utils/sessionUtils";

// Separate the component that uses useSearchParams
function AuthHandler() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const session = searchParams.get('session');

  useEffect(() => {
    const handleAsync = async () => {
      if (session) {
        try {
          // Decode base64 session parameter
          const decodedSession = atob(session);
          storeSessionFromString(decodedSession);
          await router.push("/");
        } catch (error) {
          console.error('Failed to decode session:', error);
        }
      } else {
        // Check if user is already authenticated via getSession
        try {
          const existingSession = getSession();

          if (existingSession) {
            // User is already authenticated, redirect to /
            await router.push("/");
          }
        } catch (error) {
          console.error('Failed to get existing session:', error);
          // Could potentially redirect to login page or show an error
        }
      }
    };
    handleAsync();
  }, [session, router]);

  return null; // This component only handles the auth logic
}

export default function Page() {
  return (
    <>
      <title>Authenticating with Github</title>
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
              <h1 className="text-3xl font-semibold mb-6">Authenticating...</h1>
              <p className="flex justify-center">
                <svg className="animate-spin -ml-1 mr-3 h-5 w-5"
                     xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10"
                          stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Please wait...
              </p>
            </div>
          </Container>
        </main>
        <Footer/>
        {/* Wrap the component that uses useSearchParams in Suspense */}
        <Suspense fallback={null}>
          <AuthHandler />
        </Suspense>
      </div>
    </>
  )
}

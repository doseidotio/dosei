import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { storeContinuePath } from "@/utils/sessionUtils";

export default function useIsAuthenticated(redirect = false) {

  const router = useRouter();
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    // Ensure router is available
    if (!router) {
      console.warn('Router not available');
      return;
    }
    const auth = !!localStorage.getItem("session");
    setIsAuthenticated(auth);
    if (redirect && !auth) {
      storeContinuePath(router.asPath);
      router.push("/login");
    }
  }, [redirect, router])

  return isAuthenticated;
}

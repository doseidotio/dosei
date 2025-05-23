'use client'

import useIsAuthenticated from "@/hooks/useIsAuthenticated";
import { Container } from "@/components/v2/Container";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useRouter } from "next/navigation";
import { useEffect } from "react";

export default function Home() {
  useIsAuthenticated(true);
  const router = useRouter();

  const userQuery = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const response = await api.get("/user");
      return await response.json();
    },
  });

  useEffect(() => {
    if (userQuery.isSuccess && userQuery.data?.name) {
      router.push(`/${userQuery.data.name}`);
    }
  }, [userQuery.isSuccess, userQuery.data, router]);

  if (userQuery.isLoading || !userQuery.data) return null;

  return null;
}

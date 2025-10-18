import { OpenApp, commands } from "@/types/tauri.gen";
import { useQuery } from "@tanstack/react-query";

export const useOpenApps = () => {
  const {
    data: apps = [],
    isLoading,
    error,
    refetch: fetch,
  } = useQuery({
    queryKey: ["openApps"],
    queryFn: (): Promise<OpenApp[]> => {
      return commands.getOpenApps();
    },
    refetchOnWindowFocus: false,
    staleTime: Infinity,
  });

  return {
    apps,
    loading: isLoading,
    error,
    fetch,
  };
};

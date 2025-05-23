'use client';

import ConsoleLayout from "@/app/ConsoleLayout";
import {AccountNav} from "@/components/AccountNav";
import {useQuery} from "@tanstack/react-query";
import {api} from "@/lib/api";

export default function AccountPage() {

  const userQuery = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const response = await api.get("/user");
      return await response.json();
    },
  });

  if (userQuery.isLoading || !userQuery.data) return false;

  return (
    <>
      <ConsoleLayout>
        <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
          <div className="flex justify-between">
            <h2 className="text-3xl font-medium text-gray-900 dark:text-white mb-8 flex items-center">
              General
            </h2>
          </div>
          <div className="space-y-6 lg:px-0 lg:col-span-9">
            <div className="lg:grid lg:grid-cols-12 lg:gap-x-5">
              <AccountNav/>

              <div className="space-y-6 sm:px-6 lg:px-0 lg:col-span-9">
                <form>
                  <div className="bg-white dark:bg-neutral-900 rounded-md border dark:border-neutral-800 sm:overflow-hidden">
                    <div className="py-6 px-4 space-y-6 sm:p-6">
                      <div>
                        <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-white">
                          Account
                        </h3>
                        <p className="text-sm font-medium text-gray-500 dark:text-neutral-400">
                          Unique account identifier.
                        </p>
                      </div>
                      <div className="mt-2">
                        <div
                          className="flex rounded-md ring-1 dark:ring-neutral-800 ring-inset ring-gray-300 focus-within:ring-2 focus-within:ring-inset focus-within:ring-blue-600 sm:max-w-md">
                          <span
                            className="flex select-none items-center pl-3 text-gray-500 dark:text-neutral-400 sm:text-sm">dosei.io/</span>
                          <input
                            disabled
                            type="text"
                            name="project-name"
                            id="project-name"
                            className="block flex-1 border-0 bg-transparent py-1.5 pl-1 text-gray-900 dark:text-white placeholder:text-gray-400 focus:ring-0 sm:text-sm sm:leading-6"
                            value={userQuery.data.name}
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>
      </ConsoleLayout>
    </>
  )
}

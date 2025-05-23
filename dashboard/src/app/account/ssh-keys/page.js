'use client';

import ConsoleLayout from "@/app/ConsoleLayout";
import {KeyIcon} from "@heroicons/react/24/outline";
import {useState} from "react";
import {AccountNav} from "@/components/AccountNav";
import moment from 'moment';
import {useQuery} from "@tanstack/react-query";
import {api} from "@/lib/api";

const SSHKeyItem = ({ sshKey }) => {
  return (
    <li className="hover:bg-gray-50 dark:hover:bg-white/10">
      <div className="px-4 py-4 sm:px-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <KeyIcon className="h-5 w-5 text-gray-400 dark:text-white/60" aria-hidden="true" />
            </div>
            <div className="ml-3 min-w-0">
              <div className="mt-1 flex items-center space-x-4">
                <p className="text-sm text-gray-500 dark:text-white/50 font-mono">
                  {sshKey.key_fingerprint}<br/>
                </p>
              </div>
            </div>
          </div>
          <div className="ml-4">
                              <span className="text-sm text-gray-500 dark:text-white/50">
                    Added on {moment(sshKey.created_at).format('MMM D, YYYY')}
                  </span>
          </div>
        </div>
      </div>
    </li>
  );
};

export default function SSHKeysPage() {

  const sshKeysQuery = useQuery({
    queryKey: ['user/ssh-key'],
    queryFn: async () => {
      const response = await api.get("/user/ssh-key");
      return await response.json();
    },
  });

  if (sshKeysQuery.isLoading || !sshKeysQuery.data) return false;

  return (
    <>
      <ConsoleLayout>
        <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
          <div className="flex justify-between">
            <h2 className="text-3xl font-medium text-gray-900 dark:text-white mb-8 flex items-center">
              SSH keys
            </h2>
          </div>

          <div className="space-y-6 lg:px-0 lg:col-span-9">
            <div className="lg:grid lg:grid-cols-12 lg:gap-x-5">
              <AccountNav/>

              <div className="space-y-6 sm:px-6 lg:px-0 lg:col-span-9">
                <div className="bg-white dark:bg-neutral-900 rounded-md border dark:border-neutral-800 overflow-hidden">
                  <ul className="divide-y divide-gray-200 dark:divide-neutral-800">
                    {sshKeysQuery.data.map((sshKey) => (
                      <SSHKeyItem
                        key={sshKey.id}
                        sshKey={sshKey}
                      />
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
      </ConsoleLayout>
    </>
  )
}

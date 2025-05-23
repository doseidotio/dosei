import React, { Fragment } from "react";
import {useRouter} from "next/navigation";
import clsx from "clsx";
import Link from "next/link";
import { deleteSession } from "@/utils/sessionUtils";
import {useMutation, useQuery} from "@tanstack/react-query";
import { Menu, Transition } from "@headlessui/react";
import {api} from "@/lib/api";

const ProfileMenu = () => {
  const router = useRouter();
  const logoutMutation = useMutation({
    mutationFn: () => api.delete("/auth/logout"),
    onSettled: async (data, error, variables, context) => {
      deleteSession();
      // queryClient.clear();
      await router.push("/login");
    },
  })
  return (
    <>
      <Menu as="div" className="relative">
        <div className="px-2 flex">
          <Menu.Button
            className={clsx(
              "hover:text-gray-900 hover:bg-gray-100",
              "dark:hover:bg-white/10",
              "group flex flex-1 items-center p-2 rounded-md",
            )}>
            <div className="relative">
              <img
                src="/profile.png"
                className="border h-9 w-9 rounded-full"
              />
            </div>
          </Menu.Button>
        </div>
        <Transition
          as={Fragment}
          enter="transition ease-out duration-100"
          enterFrom="transform opacity-0 scale-95"
          enterTo="transform opacity-100 scale-100"
          leave="transition ease-in duration-75"
          leaveFrom="transform opacity-100 scale-100"
          leaveTo="transform opacity-0 scale-95"
        >
          <Menu.Items className={clsx(
            "absolute mt-2 w-52 rounded-md py-1 bg-white ring-1 ring-black ring-opacity-5 focus:outline-none right-0",
            "dark:bg-neutral-900 dark:bg-black-10 dark:ring-white/10",
          )}>
            {/*<Menu.Item>*/}
            {/*  <Link className={clsx(*/}
            {/*    'block px-4 py-2 text-sm',*/}
            {/*    "text-gray-700 hover:bg-gray-100",*/}
            {/*    "dark:text-white/70 dark:hover:bg-white/20"*/}
            {/*  )} href="/profile">*/}
            {/*    Profile*/}
            {/*  </Link>*/}
            {/*</Menu.Item>*/}
            <Menu.Item>
              <div
                className={clsx(
                  'block px-4 py-2 text-sm cursor-pointer',
                  "text-gray-700 hover:bg-gray-100 ",
                  "dark:text-white/70 dark:hover:bg-white/20"
                )}
                onClick={() => logoutMutation.mutate()}
              >
                Log out
              </div>
            </Menu.Item>
          </Menu.Items>
        </Transition>
      </Menu>
    </>
  );
};

export default ProfileMenu;

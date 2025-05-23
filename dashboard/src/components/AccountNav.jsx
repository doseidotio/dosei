import React from "react";
import Link from "next/link";
import {KeyIcon, InformationCircleIcon} from "@heroicons/react/24/outline";
import clsx from "clsx";
import {useParams, usePathname} from "next/navigation";

export const AccountNav = () => {
  const params = useParams();
  const pathname = usePathname();
  const accountNav = [
    {
      name: "General",
      href: () => `/account`,
      icon: InformationCircleIcon
    },
    {
      name: "SSH Keys",
      href: () => `/account/ssh-keys`,
      icon: KeyIcon
    },
  ];
  return (
    <aside className="py-6 lg:py-0 lg:col-span-3">
      <nav className="space-y-1">
        {
          accountNav.map((item) => {
            const current = item.href() === pathname;
            return (
              <Link
                className={clsx(
                  current ? "bg-white/10 font-semibold" : "",
                  "group rounded-md px-3 py-2 flex items-center text-sm font-medium hover:bg-white/15",
                )}
                aria-current={current ? "page" : undefined}
                key={item.name}
                href={item.href()}
              >
                <item.icon
                  className={clsx(
                    current
                      ? "text-white"
                      : "text-white",
                    "flex-shrink-0 -ml-1 mr-3 h-6 w-6",
                  )}
                  aria-hidden="true"
                />
                <span className="truncate">{item.name}</span>
              </Link>
            );
          })
        }
      </nav>
    </aside>
  );
};

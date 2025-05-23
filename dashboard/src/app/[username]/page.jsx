'use client'

import React from "react";
import { Container } from "@/components/v2/Container";
import {useQuery} from "@tanstack/react-query";
import { BookOpenIcon, ServerIcon } from "@heroicons/react/24/outline";
import {PrimaryButton} from "@/components/v2/PrimaryButton";
import ConsoleLayout from "@/app/ConsoleLayout";
import {api} from "@/lib/api";
import {ChevronRightIcon} from "@heroicons/react/20/solid";
import Link from "next/link";
import moment from "moment";

export default function Home() {

  const userQuery = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const response = await api.get("/user");
      return await response.json();
    },
  });

  const servicesQuery = useQuery({
    queryKey: ['service'],
    queryFn: async () => {
      const response = await api.get("/service");
      return await response.json();
    },
  });

  if (servicesQuery.isLoading || userQuery.isLoading || !servicesQuery.data || !userQuery.data) return false;

  return (
    <ConsoleLayout>
      <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
        <div className="flex justify-between">
          <h2 className="text-3xl font-medium text-gray-900 dark:text-white mb-8 flex items-center">
            Services
          </h2>
        </div>
        <div className="space-y-6 lg:px-0 lg:col-span-9">
          <div className="overflow-hidden border rounded-md border-neutral-800">
            <div className="overflow-hidden bg-white dark:bg-neutral-900">
              {servicesQuery.isLoading ? (
                <ul role="list" className="divide-y divide-gray-200 dark:divide-neutral-800">
                  {[...Array(3).keys()].map(key => <ProjectRowSkeleton key={key}/>)}
                </ul>
              ) : servicesQuery.data.length === 0 ? (
                <div className="text-center py-12 px-4">
                  <ServerIcon className="mx-auto h-12 w-12 text-gray-400 dark:text-neutral-500 mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    No services yet
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-neutral-400 mb-6 max-w-sm mx-auto">
                    Get started by creating your first service.
                  </p>
                  <PrimaryButton
                    icon={BookOpenIcon}
                    title="How to create a service"
                    target="_blank"
                    href={process.env.NEXT_PUBLIC_DOCS_BASE_URL + "/getting-started/init-a-service"}
                  />
                </div>
              ) : (
                <ul role="list" className="divide-y divide-gray-200 dark:divide-neutral-800">
                  {servicesQuery.data.map((service) => <ProjectRow key={service.id}
                                                                   username={userQuery.data.name}
                                                                   service={service}/>)}
                </ul>
              )}
            </div>
          </div>
        </div>
      </div>
    </ConsoleLayout>
  );
}

export const ProjectRowSkeleton = () => {
  return (
    <div className="flex items-center px-4 py-4 sm:px-6">
      <div className="flex min-w-0 flex-1 items-center">
        <div className="min-w-0 flex-1 grid-col-2 md:grid md:grid-cols-3 md:gap-4">
          <div className="space-y-2">
            <div className="h-4 bg-gray-900 dark:bg-neutral-800 rounded col-span-2 w-32"/>
            <div className="h-4 bg-gray-200 dark:bg-neutral-800 rounded col-span-2 w-36"/>
          </div>
          <div className="hidden px-14 md:flex items-center">
            <div className="h-10 bg-gray-200 dark:bg-neutral-800 rounded w-72"/>
          </div>
          <div className="space-y-2">
            <div className="h-4 bg-gray-200 dark:bg-neutral-800 rounded col-span-2 w-56"/>
            <div className="h-4 bg-gray-200 dark:bg-neutral-800 rounded col-span-2 w-72"/>
          </div>
        </div>
      </div>
      <div>
        <ChevronRightIcon className="h-5 w-5 text-gray-400 dark:text-neutral-600" aria-hidden="true" />
      </div>
    </div>
  );
};

export const ProjectRow = ({ username, service }) => {

  const serviceIngressQuery = useQuery({
    queryKey: [`service/${service.id}/ingress`],
    queryFn: async () => {
      const response = await api.get(`/service/${service.id}/ingress`);
      return await response.json();
    },
  });

  const serviceDeploymentsQuery = useQuery({
    queryKey: [`service/${service.id}/deployment`],
    queryFn: async () => {
      const response = await api.get(`/service/${service.id}/deployment`);
      return await response.json();
    },
  });

  if (serviceIngressQuery.isLoading || !serviceIngressQuery.data || serviceDeploymentsQuery.isLoading || !serviceDeploymentsQuery.data) {
    return <ProjectRowSkeleton/>;
  }

  let lastAccessedAt = serviceDeploymentsQuery.data[0].last_accessed_at;
  let fLastAccessedAt = lastAccessedAt
    ? moment(lastAccessedAt).subtract(5, 'seconds').fromNow()
    : "Unknown";

  return (
    <li>
      <div className="block hover:bg-gray-50 dark:bg-neutral-900 dark:hover:bg-white/10">
        <div className="flex items-center px-4 py-4 sm:px-6">
          <div className="flex min-w-0 flex-1 items-center">
            <div className="min-w-0 flex-1 grid-col-2 md:grid md:grid-cols-3 md:gap-4">
              <div>
                <p className="text-sm text-gray-900 dark:text-white flex">
                  <div className="flex items-center justify-center w-5 h-5 bg-green-100 dark:bg-green-900/30 rounded-full">
                    <div className="w-2 h-2 bg-green-600 dark:bg-green-400 rounded-full animate-pulse"></div>
                  </div>
                  <span className="ml-2">{service.name}</span>
                </p>
                {/*Standby one*/}
                {/*<p className="text-sm text-gray-900 dark:text-white flex">*/}
                {/*  <div className="flex items-center justify-center w-5 h-5 bg-orange-100 dark:bg-orange-900/30 rounded-full">*/}
                {/*    <div className="w-2 h-2 bg-orange-600 dark:bg-orange-400 rounded-full"></div>*/}
                {/*  </div>*/}
                {/*  <span className="ml-2">{service.name}</span>*/}
                {/*</p>*/}
                {
                  serviceIngressQuery.data.length > 0 &&
                  <p className="mt-2 flex items-center text-sm text-gray-500 dark:text-white/50">
                    {serviceIngressQuery.data.map((ingress, index) => (
                      <span key={index}>
        <Link
          href={`https://${ingress.host}`}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:underline hover:text-blue-600 dark:hover:text-blue-400"
        >
          {ingress.host}
        </Link>
                        {index < serviceIngressQuery.data.length - 1 && ', '}
      </span>
                    ))}
                  </p>
                }
              </div>
              <div className="hidden px-14 md:flex items-center">
                {
                  serviceDeploymentsQuery.data.length > 0 &&
                  <p className="mt-2 flex items-center text-sm text-gray-500 dark:text-white/50">
                    Port: {serviceDeploymentsQuery.data[0].container_port}<br/>
                    Last Accessed {fLastAccessedAt}
                  </p>
                }
              </div>
              <div>
                {/*{*/}
                {/*  includesProduction &&*/}
                {/*  <div className="flex gap-x-1 items-center text-gray-500 dark:text-white/50 text-sm leading-6">*/}
                {/*    <dt>*/}
                {/*      <span className="text-gray-600">{deploymentQuery.data.git.commit.message}</span> on <span className="text-gray-600">{deploymentQuery.data.git.branch}</span>*/}
                {/*    </dt>*/}
                {/*    <svg fill="currentColor" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg" className="h-5 mr-2">*/}
                {/*      <path d="M416,160a64,64,0,1,0-96.27,55.24c-2.29,29.08-20.08,37-75,48.42-17.76,3.68-35.93,7.45-52.71,13.93V151.39a64,64,0,1,0-64,0V360.61a64,64,0,1,0,64.42.24c2.39-18,16-24.33,65.26-34.52,27.43-5.67,55.78-11.54,79.78-26.95,29-18.58,44.53-46.78,46.36-83.89A64,64,0,0,0,416,160ZM160,64a32,32,0,1,1-32,32A32,32,0,0,1,160,64Zm0,384a32,32,0,1,1,32-32A32,32,0,0,1,160,448ZM352,192a32,32,0,1,1,32-32A32,32,0,0,1,352,192Z"/>*/}
                {/*    </svg>*/}
                {/*  </div>*/}
                {/*}*/}
                {/*{*/}
                {/*  includesProduction &&*/}
                {/*  <div className="flex gap-x-1 items-center text-gray-500 dark:text-white/50">*/}
                {/*    <dt className="text-sm leading-6">*/}
                {/*      Last updated <span className="text-gray-600">{moment(deploymentQuery.data.updated_at).fromNow()}</span> via*/}
                {/*    </dt>*/}
                {/*    <svg fill="currentColor" viewBox="0 0 24 24" className="h-5 mr-2" aria-hidden="true">*/}
                {/*      <path*/}
                {/*        fillRule="evenodd"*/}
                {/*        d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"*/}
                {/*        clipRule="evenodd"*/}
                {/*      />*/}
                {/*    </svg>*/}
                {/*  </div>*/}
                {/*}*/}
                {/*<div className="flex gap-x-1 items-center text-gray-500 dark:text-white/50">*/}
                {/*  <dt className="text-sm leading-6">*/}
                {/*    Last updated <span className="text-gray-600">{moment(service.updated_at).fromNow()}</span> via*/}
                {/*  </dt>*/}
                {/*</div>*/}
              </div>
            </div>
          </div>
          {/*<div>*/}
          {/*  <ChevronRightIcon className="h-5 w-5 text-gray-400 dark:text-white/60" aria-hidden="true" />*/}
          {/*</div>*/}
        </div>
      </div>
    </li>
  );
};

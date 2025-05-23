'use client'

import React, {useState} from "react";
import { Container } from "@/components/v2/Container";
import {useQuery} from "@tanstack/react-query";
import ConsoleLayout from "@/app/ConsoleLayout";
import {api} from "@/lib/api";
import Link from "next/link";
import moment from "moment";
import {ProjectRowSkeleton} from "@/app/[username]/page";

export default function Home() {

  const certificatesQuery = useQuery({
    queryKey: ['certificates'],
    queryFn: async () => {
      const response = await api.get("/certificate");
      return await response.json();
    },
  });

  if (certificatesQuery.isLoading || !certificatesQuery.data) return false;

  return (
    <ConsoleLayout>
      <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
        <div className="flex justify-between">
          <h2 className="text-3xl font-medium text-gray-900 dark:text-white mb-8 flex items-center">
            Certificates
          </h2>
        </div>
        <div className="space-y-6 lg:px-0 lg:col-span-9">
          <div className="border rounded-md border-neutral-800">
            <div className="bg-white dark:bg-neutral-900">
              <ul role="list" className="divide-y divide-gray-200 dark:divide-neutral-800">
                {
                  certificatesQuery.isLoading ? [...Array(3).keys()].map(key => <ProjectRowSkeleton key={key}/>) :
                    certificatesQuery.data.map((certificate) => (
                      <CertificateRow key={certificate.id} certificate={certificate} />
                    ))
                }
              </ul>
            </div>
          </div>
        </div>
      </div>
    </ConsoleLayout>
  );
}

export const CertificateRow = ({ certificate }) => {
  const [showTooltip, setShowTooltip] = useState(false);
  const expiresAt = moment(certificate.expires_at);

  const handleDownload = (e) => {
    e.preventDefault();
    e.stopPropagation();

    // In a real implementation, you would call an API endpoint to download the certificate
    // This is just a placeholder for the actual download functionality
    window.alert("Not implemented :)");
  };

  return (
    <li className="relative overflow-visible">
      <div className="block hover:bg-gray-50 dark:bg-neutral-900 dark:hover:bg-white/10">
        <div className="flex items-center px-4 py-4 sm:px-6">
          <div className="flex min-w-0 flex-1 items-center">
            <div className="min-w-0 flex-1 grid-col-2 md:grid md:grid-cols-3 md:gap-4">
              <div className="hidden md:flex items-center">
                <p className="text-sm font-medium text-gray-600 dark:text-white">
                  {certificate.domain_name}
                </p>
              </div>
              <div className="hidden md:flex items-center">
                <p className="text-sm font-medium text-gray-600 dark:text-white">
                  Created <span className="text-gray-600 dark:text-white/70">{moment(certificate.created_at).fromNow()}</span>
                </p>
              </div>
              <div className="hidden md:flex items-center space-x-2 relative overflow-visible">
                <div
                  className="relative"
                  onMouseEnter={() => setShowTooltip(true)}
                  onMouseLeave={() => setShowTooltip(false)}
                >
                  <div className="flex items-center justify-center w-5 h-5 bg-green-100 dark:bg-green-900/30 rounded-full">
                    <svg
                      className="w-3 h-3 text-green-600 dark:text-green-400"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path fillRule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clipRule="evenodd" />
                    </svg>
                  </div>

                  {/* Tooltip right next to the icon */}
                  {showTooltip && (
                    <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 z-50">
                      <div className="bg-gray-900 dark:bg-gray-700 text-white text-xs rounded-lg px-3 py-2 whitespace-nowrap shadow-lg">
                        Auto-renewal enabled
                        <div className="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900 dark:border-t-gray-700"></div>
                      </div>
                    </div>
                  )}
                </div>
                <p className="text-sm font-medium text-gray-600 dark:text-white">
                  Expires <span className="text-gray-600 dark:text-white/70">{expiresAt.format('MMM D, YYYY')}</span>
                </p>
              </div>
            </div>
          </div>
          <div className="flex items-center space-x-3">
            <button
              onClick={handleDownload}
              className="inline-flex items-center rounded-md bg-white px-2.5 py-1.5 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 dark:bg-gray-800 dark:text-white dark:ring-gray-600 dark:hover:bg-gray-700"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Download
            </button>
          </div>
        </div>
      </div>

      {/* Removed the separate tooltip positioning */}
    </li>
  );
};

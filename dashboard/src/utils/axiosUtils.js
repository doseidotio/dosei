import axios from "axios";
import { getApiBaseURL } from "@/utils/sessionUtils";

export const axiosInstance = axios.create({
  baseURL: getApiBaseURL()
});

axiosInstance.interceptors.request.use((config) => {
  const token = localStorage.getItem("token");
  if (token) {
    config.headers.Authorization = "Bearer " + token;
  }
  return config;
});

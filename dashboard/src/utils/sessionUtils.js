export const storeSession = (session) => {
  localStorage.setItem('session', JSON.stringify(session));
};

export const storeSessionFromString = (session) => {
  localStorage.setItem('session', session);
};

export const getSession = () => {
  return localStorage.getItem("session");
};

export const deleteSession = () => {
  localStorage.removeItem("session");
};


export const storeContinuePath = (continue_uri) => {
  localStorage.setItem("continue_path", continue_uri);
};

export const getContinuePath = () => {
  return localStorage.getItem("continue_path");
};

export const getContinuePathAndDelete = () => {
  const continueURI = localStorage.getItem("continue_path");
  localStorage.removeItem("continue_path");
  return continueURI;
};

export const getApiBaseURL = () => {
  if (typeof window !== 'undefined') {
    const apiBaseURL = localStorage.getItem("api_base_url");
    if (apiBaseURL) {
      return apiBaseURL;
    }
  }
  return process.env.NEXT_PUBLIC_API_BASE_URL;
};

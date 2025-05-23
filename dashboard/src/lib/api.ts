import { getApiBaseURL } from "@/utils/sessionUtils";

interface Session {
  id: string,
  token: string,
  refresh_token: string
}

export enum HttpMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  PATCH = 'PATCH',
  DELETE = 'DELETE'
}

export enum ContentType {
  APPLICATION_JSON = 'application/json'
}

export class api {

  private static async request(
    method: HttpMethod = HttpMethod.GET,
    url: string,
    body?: BodyInit | null,
    contentType?: ContentType | null,
  ) {
    let token = "";
    const sessionData = localStorage.getItem('session');

    if (sessionData) {
      try {
        const session: Session = JSON.parse(sessionData);
        token = session.token;
      } catch (error) {
        console.error('Failed to parse session data:', error);
        localStorage.removeItem('session');
      }
    }

    const headers: Record<string, string> = {
      'Authorization': `Bearer ${token}`,
    };
    if (contentType) {
      headers['Content-Type'] = contentType;
    }

    return await fetch(`${getApiBaseURL()}${url}`, {
      method,
      headers,
      body
    });
  }

  static async get(
    url: string,
    body?: BodyInit | null,
    contentType?: ContentType | null
  ) : Promise<Response> {
    return await this.request(HttpMethod.GET, url, body, contentType);
  }

  static async post(
    url: string,
    body?: BodyInit | null,
    contentType?: ContentType | null
  ) : Promise<Response> {
    return await this.request(HttpMethod.POST, url, body, contentType);
  }

  static async put(
    url: string,
    body?: BodyInit | null,
    contentType?: ContentType | null
  ) : Promise<Response> {
    return await this.request(HttpMethod.PUT, url, body, contentType);
  }

  static async patch(
    url: string,
    body?: BodyInit | null,
    contentType?: ContentType | null
  ) : Promise<Response> {
    return await this.request(HttpMethod.PATCH, url, body, contentType);
  }

  static async delete(
    url: string,
    body?: BodyInit | null,
    contentType?: ContentType | null
  ) : Promise<Response> {
    return await this.request(HttpMethod.DELETE, url, body, contentType);
  }
}

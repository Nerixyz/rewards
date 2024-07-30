import { ref } from 'vue';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyObject = Record<string, any>;

/**
 * The foundation for the ApiClient. It's just to separate the core functions from the specific API functions.
 */
export class BaseClient {
  private authToken?: string = getToken();

  isAuthenticated = ref(!!this.authToken);

  logout(): void {
    this.authToken = undefined;
    localStorage.removeItem('authToken');
    this.isAuthenticated.value = false;
  }

  protected get<T>(...segments: string[]): Promise<T> {
    return this.baseRequest(segments.join('/'), {});
  }

  protected put<T>(data: AnyObject | undefined, ...segments: string[]): Promise<T> {
    return this.baseRequest(segments.join('/'), {
      method: 'PUT',
      body: (data && JSON.stringify(data)) ?? null,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  protected patch<T>(data: AnyObject, ...segments: string[]): Promise<T> {
    return this.baseRequest(segments.join('/'), {
      method: 'PATCH',
      body: JSON.stringify(data),
      headers: { 'Content-Type': 'application/json' },
    });
  }

  protected async delete(...segments: string[]): Promise<void> {
    await this.baseRequest(segments.join('/'), { method: 'DELETE' });
  }

  private async baseRequest<T>(url: string, opts: RequestInit): Promise<T> {
    const response = await fetch(makeApiUrl(url), {
      ...opts,
      headers: {
        ...opts.headers,
        Authorization: this.authToken ? `Bearer ${this.authToken}` : '',
      },
    });
    if (response.headers.get('content-type')?.startsWith('application/json')) {
      const json = await response.json();

      if (isOk(response.status)) return json;

      if (response.status === 401 && this.isAuthenticated.value) this.logout();
      throw new Error(json.error ?? 'An error occurred.');
    } else {
      const text = await response.text();

      if (isOk(response.status)) return text as unknown as T;

      if (response.status === 401 && this.isAuthenticated.value) this.logout();
      throw new Error(text ?? 'An error occurred.');
    }
  }
}

function makeApiUrl(path: string) {
  return `${import.meta.env.MODE === 'development' ? (import.meta.env['VITE_API_BASE_URL'] ?? '') : ''}/api/v1/${path}`;
}

function isOk(status: number) {
  return status >= 200 && status < 300;
}

function getToken() {
  let cookie: string | undefined | null = localStorage.getItem('authToken');
  if (cookie) return cookie;

  cookie = document.cookie.match(/auth_token=([^;]+)/)?.[1];
  if (!cookie) return;

  localStorage.setItem('authToken', cookie);
  document.cookie = 'auth_token=;expires=0;SameSite=None; Secure';

  return cookie;
}

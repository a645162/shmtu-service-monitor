import axios from 'axios';
import type {
  Service,
  ServiceStatus,
  CreateServiceRequest,
  DashboardSummary,
} from '../types';

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
});

// ── Services ──

export async function listServices(): Promise<Service[]> {
  const res = await api.get('/services');
  return res.data;
}

export async function getService(id: number): Promise<Service> {
  const res = await api.get(`/services/${id}`);
  return res.data;
}

export async function registerService(data: CreateServiceRequest): Promise<Service> {
  const res = await api.post('/services', data);
  return res.data;
}

export async function deleteService(id: number): Promise<void> {
  await api.delete(`/services/${id}`);
}

// ── Status ──

export async function getServiceStatus(id: number): Promise<ServiceStatus> {
  const res = await api.get(`/services/${id}/status`);
  return res.data;
}

export async function getServiceHistory(
  id: number,
  params?: { from?: string; to?: string; limit?: number }
): Promise<ServiceStatus[]> {
  const res = await api.get(`/services/${id}/history`, { params });
  return res.data;
}

// ── Dashboard ──

export async function getDashboard(): Promise<DashboardSummary> {
  const res = await api.get('/dashboard');
  return res.data;
}

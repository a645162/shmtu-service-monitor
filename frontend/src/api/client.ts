import axios from 'axios';
import type {
  MonitorServer,
  ServiceInstance,
  ServiceStatus,
  CreateServerRequest,
  CreateInstanceRequest,
  DashboardSummary,
  ServerDetail,
} from '../types';

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
});

// ── Servers ──

export async function listServers(): Promise<MonitorServer[]> {
  const res = await api.get('/servers');
  return res.data;
}

export async function getServer(id: number): Promise<MonitorServer> {
  const res = await api.get(`/servers/${id}`);
  return res.data;
}

export async function createServer(data: CreateServerRequest): Promise<MonitorServer> {
  const res = await api.post('/servers', data);
  return res.data;
}

export async function deleteServer(id: number): Promise<void> {
  await api.delete(`/servers/${id}`);
}

export async function getServerDetail(id: number): Promise<ServerDetail> {
  const res = await api.get(`/servers/${id}/detail`);
  return res.data;
}

// ── Instances ──

export async function listInstances(serverId: number): Promise<ServiceInstance[]> {
  const res = await api.get(`/servers/${serverId}/instances`);
  return res.data;
}

export async function getInstance(id: number): Promise<ServiceInstance> {
  const res = await api.get(`/instances/${id}`);
  return res.data;
}

export async function registerInstance(serverId: number, data: CreateInstanceRequest): Promise<ServiceInstance> {
  const res = await api.post(`/servers/${serverId}/instances`, data);
  return res.data;
}

export async function deleteInstance(id: number): Promise<void> {
  await api.delete(`/instances/${id}`);
}

// ── Status ──

export async function getInstanceStatus(id: number): Promise<ServiceStatus> {
  const res = await api.get(`/instances/${id}/status`);
  return res.data;
}

export async function getInstanceHistory(
  id: number,
  params?: { from?: string; to?: string; limit?: number }
): Promise<ServiceStatus[]> {
  const res = await api.get(`/instances/${id}/history`, { params });
  return res.data;
}

// ── Dashboard ──

export async function getDashboard(): Promise<DashboardSummary> {
  const res = await api.get('/dashboard');
  return res.data;
}

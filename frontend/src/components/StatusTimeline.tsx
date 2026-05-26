import React from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
} from 'recharts';
import { Empty } from 'antd';
import dayjs from 'dayjs';
import type { ServiceStatus } from '../types';

interface Props {
  history: ServiceStatus[];
}

const StatusTimeline: React.FC<Props> = ({ history }) => {
  if (history.length === 0) {
    return <Empty description="No history data" />;
  }

  const chartData = [...history].reverse().map((st) => ({
    time: dayjs(st.polled_at).format('HH:mm:ss'),
    response_ms: Number(st.response_time_ms.toFixed(1)),
    utilization: Number(st.utilization_percent.toFixed(1)),
    pending: st.pending_requests,
    status: st.status,
  }));

  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="time" />
        <YAxis yAxisId="left" />
        <YAxis yAxisId="right" orientation="right" />
        <Tooltip />
        <Legend />
        <Line
          yAxisId="left"
          type="monotone"
          dataKey="response_ms"
          stroke="#1677ff"
          name="Response (ms)"
          dot={false}
        />
        <Line
          yAxisId="right"
          type="monotone"
          dataKey="utilization"
          stroke="#faad14"
          name="Utilization (%)"
          dot={false}
        />
      </LineChart>
    </ResponsiveContainer>
  );
};

export default StatusTimeline;

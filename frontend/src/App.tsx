import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider, App as AntApp, theme } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import Dashboard from './components/Dashboard';
import ServerList from './components/ServerList';
import ServerDetailView from './components/ServerDetail';
import InstanceDetail from './components/InstanceDetail';

const App: React.FC = () => (
  <ConfigProvider
    locale={zhCN}
    theme={{
      algorithm: theme.defaultAlgorithm,
      token: {
        colorPrimary: '#1677ff',
      },
    }}
  >
    <AntApp>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/servers" element={<ServerList />} />
          <Route path="/servers/:id" element={<ServerDetailView />} />
          <Route path="/instances/:id" element={<InstanceDetail />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </AntApp>
  </ConfigProvider>
);

export default App;

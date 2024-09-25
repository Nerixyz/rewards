import { createRouter, createWebHistory } from 'vue-router';
import ApiClient from './api/ApiClient';
const Home = () => import('./routes/Home.vue');
const NotFound = () => import('./routes/NotFound.vue');
const FailedAuth = () => import('./routes/FailedAuth.vue');
const RewardsDashboard = () => import('./routes/RewardsDashboard.vue');
const EditorsDashboard = () => import('./routes/EditorsDashboard.vue');
const BroadcasterDashboard = () => import('./routes/BroadcasterDashboard.vue');
const ConnectionsDashboard = () => import('./routes/ConnectionsDashboard.vue');
const LogViewer = () => import('./routes/LogViewer.vue');
const EditReward = () => import('./routes/EditReward.vue');
const NewReward = () => import('./routes/NewReward.vue');
const TestRewardEditor = () => import('./routes/TestRewardEditor.vue');

const router = createRouter({
  routes: [
    {
      path: '/',
      component: Home,
      meta: {
        open: true,
      },
    },
    {
      path: '/failed-auth',
      component: FailedAuth,
      meta: {
        open: true,
      },
    },
    {
      name: 'Rewards',
      path: '/rewards/:id?',
      component: RewardsDashboard,
    },
    { name: 'Editors', path: '/editors', component: EditorsDashboard },
    { name: 'Broadcasters', path: '/broadcasters', component: BroadcasterDashboard },
    { name: 'Connections', path: '/connections', component: ConnectionsDashboard },
    {
      name: 'Logs',
      path: '/rewards/logs/:id?',
      component: LogViewer,
      meta: {
        ignoreNav: true,
      },
    },
    {
      name: 'New Reward',
      path: '/rewards/:id/new',
      component: NewReward,
      meta: {
        ignoreNav: true,
      },
    },
    {
      path: '/testing/editor',
      component: TestRewardEditor,
      meta: {
        open: true,
      },
    },
    {
      name: 'Edit Reward',
      path: '/rewards/:id/:rewardId',
      component: EditReward,
      meta: {
        ignoreNav: true,
      },
    },
    {
      path: '/:pathMatch(.*)*',
      component: NotFound,
      meta: {
        open: true,
      },
    },
  ],
  history: createWebHistory(),
});

router.beforeEach(to => {
  if (to.meta['open']) {
    return true;
  }
  if (!ApiClient.isAuthenticated.value) return '/';

  return true;
});
router.afterEach(to => {
  document.title = to.name?.toString() ?? 'Rewards';
});

export default router;

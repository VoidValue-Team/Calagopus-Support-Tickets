import { faHeadset } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Extension, ExtensionContext } from 'shared';
import CardSummary from './components/CardSummary.tsx';
import AccountTicketsPage from './pages/AccountTicketsPage.tsx';
import AdminTicketsPage from './pages/AdminTicketsPage.tsx';
import ConfigurationPage from './pages/ConfigurationPage.tsx';
import ServerTicketsPage from './pages/ServerTicketsPage.tsx';
import { getExtTranslations } from './translations.ts';

class SupportTicketsExtension extends Extension {
  public cardConfigurationPage: React.FC | null = ConfigurationPage;
  public cardComponent: React.FC | null = CardSummary;
  public cardIcon: React.ReactNode = <FontAwesomeIcon icon={faHeadset} />;
  public initialize(ctx: ExtensionContext): void {
    ctx.extensionRegistry.enterRoutes((routes) =>
      routes
        .addAccountRoute({
          name: () => getExtTranslations().t('nav.support', {}),
          icon: faHeadset,
          path: '/support',
          element: AccountTicketsPage,
        })
        .addAdminRoute({
          name: () => getExtTranslations().t('nav.support', {}),
          icon: faHeadset,
          path: '/support',
          element: AdminTicketsPage,
          permission: 'support.read',
        })
        .addServerRoute({
          name: () => getExtTranslations().t('nav.support', {}),
          icon: faHeadset,
          path: '/support',
          element: ServerTicketsPage,
          permission: 'support.read',
        }),
    );
    ctx.extensionRegistry.permissionIcons
      .addUserPermissionIcon('tickets', <FontAwesomeIcon icon={faHeadset} />)
      .addAdminPermissionIcon('support', <FontAwesomeIcon icon={faHeadset} />)
      .addServerPermissionIcon('support', <FontAwesomeIcon icon={faHeadset} />);
  }
}
export default new SupportTicketsExtension();

import { useEffect, useRef, useState, type RefObject } from "react";
import { NavigationContainer, type NavigationContainerRef } from "@react-navigation/native";
import { createNativeStackNavigator, type NativeStackScreenProps } from "@react-navigation/native-stack";
import * as Notifications from "expo-notifications";

import { createGatewayClient } from "../api/client";
import { AuthProvider, useAuth } from "../auth/AuthProvider";
import { color } from "../theme";
import { readRuntimeConfig } from "../config/env";
import { AssetDetailScreen } from "../screens/AssetDetailScreen";
import { AssetListScreen } from "../screens/AssetListScreen";
import { ConfigErrorScreen } from "../screens/ConfigErrorScreen";
import { ComplianceScreen } from "../screens/ComplianceScreen";
import { ConsentScreen } from "../screens/ConsentScreen";
import { HomeScreen } from "../screens/HomeScreen";
import { LoginScreen } from "../screens/LoginScreen";
import { OrganizationListScreen, type OrganizationSummary } from "../screens/OrganizationListScreen";
import { OrganizationMembersScreen } from "../screens/OrganizationMembersScreen";
import { ProjectDetailScreen } from "../screens/ProjectDetailScreen";
import { ProjectListScreen } from "../screens/ProjectListScreen";
import { UploadScreen } from "../screens/UploadScreen";
import { ReviewInboxScreen } from "../screens/ReviewInboxScreen";
import { ReviewDetailScreen } from "../screens/ReviewDetailScreen";
import type { ReviewTaskSummary } from "../api/review";
import { registerPush } from "../push/registerPush";

type UnauthedStackParamList = {
  Login: undefined;
};

type AuthedStackParamList = {
  Home: undefined;
  AssetList: undefined;
  AssetDetail: {
    assetId: string;
    assetTitle: string;
  };
  Compliance: { assetId: string; assetTitle: string };
  Consent: { assetId: string; assetTitle: string };
  Upload: undefined;
  OrganizationList: undefined;
  OrganizationMembers: { orgId: string; orgName: string; viewerRole: OrganizationSummary["viewer_role"] };
  ProjectList: { orgId: string; orgName: string };
  ProjectDetail: { orgId: string; projectId: string; projectName: string };
  ReviewInbox: { initialTaskId?: string } | undefined;
  ReviewDetail: { task: ReviewTaskSummary };
};

const UnauthedStack = createNativeStackNavigator<UnauthedStackParamList>();
const AuthedStack = createNativeStackNavigator<AuthedStackParamList>();
const AUTHTED_NAVIGATOR_OPTIONS = {
  headerShown: false,
  contentStyle: { backgroundColor: color.canvas },
} as const;

function UnauthedNavigator() {
  return (
    <UnauthedStack.Navigator screenOptions={{ headerShown: false }}>
      <UnauthedStack.Screen name="Login">
        {() => <LoginScreen />}
      </UnauthedStack.Screen>
    </UnauthedStack.Navigator>
  );
}

function HomeRoute({
  navigation,
  gatewayBaseUrl,
  dubbridgeEnv,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "Home">["navigation"];
  gatewayBaseUrl: string;
  dubbridgeEnv: string;
}) {
  return (
    <HomeScreen
      dubbridgeEnv={dubbridgeEnv}
      gatewayBaseUrl={gatewayBaseUrl}
      onOpenAssets={() => navigation.navigate("AssetList")}
      onOpenUpload={() => navigation.navigate("Upload")}
      onOpenReview={() => navigation.navigate("ReviewInbox")}
      onOpenOrganizations={() => navigation.navigate("OrganizationList")}
    />
  );
}

function AssetListRoute({
  navigation,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "AssetList">["navigation"];
  gatewayBaseUrl: string;
}) {
  return (
    <AssetListScreen
      gatewayBaseUrl={gatewayBaseUrl}
      onOpenAsset={(asset) =>
        navigation.navigate("AssetDetail", {
          assetId: asset.id,
          assetTitle: asset.title,
        })
      }
      onOpenUpload={() => navigation.navigate("Upload")}
    />
  );
}

function ReviewInboxRoute({
  navigation,
  route,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "ReviewInbox">["navigation"];
  route: NativeStackScreenProps<AuthedStackParamList, "ReviewInbox">["route"];
  gatewayBaseUrl: string;
}) {
  return (
    <ReviewInboxScreen
      gatewayBaseUrl={gatewayBaseUrl}
      initialTaskId={route.params?.initialTaskId ?? null}
      onOpenTask={(task) => navigation.navigate("ReviewDetail", { task })}
    />
  );
}

function AssetDetailRoute({
  navigation,
  route,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "AssetDetail">["navigation"];
  route: NativeStackScreenProps<AuthedStackParamList, "AssetDetail">["route"];
  gatewayBaseUrl: string;
}) {
  return (
    <AssetDetailScreen
      assetId={route.params.assetId}
      gatewayBaseUrl={gatewayBaseUrl}
      onOpenCompliance={() =>
        navigation.navigate("Compliance", {
          assetId: route.params.assetId,
          assetTitle: route.params.assetTitle,
        })
      }
    />
  );
}

function ComplianceRoute({
  navigation,
  route,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "Compliance">["navigation"];
  route: NativeStackScreenProps<AuthedStackParamList, "Compliance">["route"];
  gatewayBaseUrl: string;
}) {
  return (
    <ComplianceScreen
      assetId={route.params.assetId}
      gatewayBaseUrl={gatewayBaseUrl}
      onManageConsent={() =>
        navigation.navigate("Consent", {
          assetId: route.params.assetId,
          assetTitle: route.params.assetTitle,
        })
      }
    />
  );
}

function OrganizationListRoute({
  navigation,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "OrganizationList">["navigation"];
  gatewayBaseUrl: string;
}) {
  return (
    <OrganizationListScreen
      gatewayBaseUrl={gatewayBaseUrl}
      onOpenProjects={(organization) =>
        navigation.navigate("ProjectList", {
          orgId: organization.id,
          orgName: organization.name,
        })
      }
      onOpenMembers={(organization) =>
        navigation.navigate("OrganizationMembers", {
          orgId: organization.id,
          orgName: organization.name,
          viewerRole: organization.viewer_role,
        })
      }
    />
  );
}

function ProjectListRoute({
  navigation,
  route,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "ProjectList">["navigation"];
  route: NativeStackScreenProps<AuthedStackParamList, "ProjectList">["route"];
  gatewayBaseUrl: string;
}) {
  return (
    <ProjectListScreen
      gatewayBaseUrl={gatewayBaseUrl}
      orgId={route.params.orgId}
      onOpenProject={(project) =>
        navigation.navigate("ProjectDetail", {
          orgId: route.params.orgId,
          projectId: project.id,
          projectName: project.name,
        })
      }
    />
  );
}

function ProjectDetailRoute({
  navigation,
  route,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "ProjectDetail">["navigation"];
  route: NativeStackScreenProps<AuthedStackParamList, "ProjectDetail">["route"];
  gatewayBaseUrl: string;
}) {
  return (
    <ProjectDetailScreen
      gatewayBaseUrl={gatewayBaseUrl}
      orgId={route.params.orgId}
      projectId={route.params.projectId}
      onOpenAsset={(assetId, assetTitle) =>
        navigation.navigate("AssetDetail", { assetId, assetTitle })
      }
    />
  );
}

function ReviewDetailRoute({
  navigation,
  route,
  gatewayBaseUrl,
}: {
  navigation: NativeStackScreenProps<AuthedStackParamList, "ReviewDetail">["navigation"];
  route: NativeStackScreenProps<AuthedStackParamList, "ReviewDetail">["route"];
  gatewayBaseUrl: string;
}) {
  return (
    <ReviewDetailScreen
      task={route.params.task}
      gatewayBaseUrl={gatewayBaseUrl}
      onBack={() => navigation.navigate("ReviewInbox")}
    />
  );
}

function registerNotificationIntent(
  response: Notifications.NotificationResponse,
  authStatus: string,
  navRef: RefObject<NavigationContainerRef<AuthedStackParamList> | null>,
  setPendingDeepLink: (link: PendingDeepLink | null) => void,
) {
  const data = response.notification.request.content.data as Record<string, unknown>;
  const refEntityType = data["ref_entity_type"];
  const refEntityId = data["ref_entity_id"];

  if (refEntityType !== "review_task" || typeof refEntityId !== "string") {
    return;
  }

  if (authStatus !== "authed") {
    setPendingDeepLink({ taskId: refEntityId });
    return;
  }

  if (navRef.current?.isReady()) {
    navigateToReviewInbox(navRef.current, refEntityId);
    return;
  }

  setPendingDeepLink({ taskId: refEntityId });
}

function useNotificationDeepLinks(
  authStatus: string,
  navRef: RefObject<NavigationContainerRef<AuthedStackParamList> | null>,
  setPendingDeepLink: (link: PendingDeepLink | null) => void,
) {
  useEffect(() => {
    const subscription = Notifications.addNotificationResponseReceivedListener(
      (response) => {
        registerNotificationIntent(response, authStatus, navRef, setPendingDeepLink);
      },
    );
    return () => subscription.remove();
  }, [authStatus, navRef, setPendingDeepLink]);
}

function usePushRegistration(
  authStatus: string,
  sessionRef: string | null,
  runtimeConfig: ReturnType<typeof readRuntimeConfig>,
) {
  useEffect(() => {
    if (!runtimeConfig.ok || authStatus !== "authed") {
      return;
    }

    const client = createGatewayClient({
      gatewayBaseUrl: runtimeConfig.value.gatewayBaseUrl,
    });
    void registerPush(client, sessionRef);
  }, [authStatus, runtimeConfig, sessionRef]);
}

function resolvePendingDeepLink(
  pendingDeepLink: PendingDeepLink | null,
  authStatus: string,
  navRef: RefObject<NavigationContainerRef<AuthedStackParamList> | null>,
  setPendingDeepLink: (link: PendingDeepLink | null) => void,
) {
  if (pendingDeepLink && authStatus === "authed" && navRef.current) {
    navigateToReviewInbox(navRef.current, pendingDeepLink.taskId);
    setPendingDeepLink(null);
  }
}

function AuthedNavigator({
  gatewayBaseUrl,
  dubbridgeEnv,
}: {
  gatewayBaseUrl: string;
  dubbridgeEnv: string;
}) {
  return (
    <AuthedStack.Navigator screenOptions={AUTHTED_NAVIGATOR_OPTIONS}>
      <AuthedStack.Screen name="Home">
        {({ navigation }) => <HomeRoute navigation={navigation} gatewayBaseUrl={gatewayBaseUrl} dubbridgeEnv={dubbridgeEnv} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="AssetList">
        {({ navigation }) => <AssetListRoute navigation={navigation} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="AssetDetail">
        {({ route, navigation }) => <AssetDetailRoute navigation={navigation} route={route} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="Compliance">
        {({ route, navigation }) => <ComplianceRoute navigation={navigation} route={route} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="Consent">
        {({ route }) => <ConsentScreen assetId={route.params.assetId} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="Upload">
        {({ navigation }) => (
          <UploadScreen
            gatewayBaseUrl={gatewayBaseUrl}
            onSuccess={() => navigation.navigate("AssetList")}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="OrganizationList">
        {({ navigation }) => <OrganizationListRoute navigation={navigation} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="OrganizationMembers">
        {({ route }) => (
          <OrganizationMembersScreen
            gatewayBaseUrl={gatewayBaseUrl}
            orgId={route.params.orgId}
            viewerRole={route.params.viewerRole}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="ProjectList">
        {({ route, navigation }) => <ProjectListRoute navigation={navigation} route={route} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="ProjectDetail">
        {({ route, navigation }) => <ProjectDetailRoute navigation={navigation} route={route} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="ReviewInbox">
        {({ navigation, route }) => <ReviewInboxRoute navigation={navigation} route={route} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="ReviewDetail">
        {({ route, navigation }) => <ReviewDetailRoute navigation={navigation} route={route} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
    </AuthedStack.Navigator>
  );
}

type PendingDeepLink = { taskId: string };

function RootNavigatorContent() {
  const auth = useAuth();
  const runtimeConfig = readRuntimeConfig();
  const navRef = useRef<NavigationContainerRef<AuthedStackParamList>>(null);
  const [pendingDeepLink, setPendingDeepLink] = useState<PendingDeepLink | null>(null);
  useNotificationDeepLinks(auth.status, navRef, setPendingDeepLink);
  usePushRegistration(auth.status, auth.sessionRef, runtimeConfig);

  function onNavReady() {
    resolvePendingDeepLink(pendingDeepLink, auth.status, navRef, setPendingDeepLink);
  }

  if (!runtimeConfig.ok) {
    return <ConfigErrorScreen message={runtimeConfig.message} />;
  }

  return (
    <NavigationContainer ref={navRef} onReady={onNavReady}>
      {auth.status === "authed" ? (
        <AuthedNavigator
          dubbridgeEnv={runtimeConfig.value.dubbridgeEnv}
          gatewayBaseUrl={runtimeConfig.value.gatewayBaseUrl}
        />
      ) : (
        <UnauthedNavigator />
      )}
    </NavigationContainer>
  );
}

function navigateToReviewInbox(
  nav: NavigationContainerRef<AuthedStackParamList>,
  taskId: string,
) {
  nav.navigate("ReviewInbox", { initialTaskId: taskId });
}

export function RootNavigator() {
  return (
    <AuthProvider>
      <RootNavigatorContent />
    </AuthProvider>
  );
}

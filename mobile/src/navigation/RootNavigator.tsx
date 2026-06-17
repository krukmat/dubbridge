import { useEffect, useRef, useState } from "react";
import { NavigationContainer, type NavigationContainerRef } from "@react-navigation/native";
import { createNativeStackNavigator } from "@react-navigation/native-stack";
import * as Notifications from "expo-notifications";

import { createGatewayClient } from "../api/client";
import { AuthProvider, useAuth } from "../auth/AuthProvider";
import { color, type } from "../theme";
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

function UnauthedNavigator() {
  return (
    <UnauthedStack.Navigator screenOptions={{ headerShown: false }}>
      <UnauthedStack.Screen name="Login">
        {() => <LoginScreen />}
      </UnauthedStack.Screen>
    </UnauthedStack.Navigator>
  );
}

function AuthedNavigator({
  gatewayBaseUrl,
  dubbridgeEnv,
}: {
  gatewayBaseUrl: string;
  dubbridgeEnv: string;
}) {
  return (
    <AuthedStack.Navigator
      screenOptions={{
        headerStyle: { backgroundColor: color.raised },
        headerTintColor: color.primary,
        headerTitleStyle: { ...type.heading, color: color.ink900 },
      }}
    >
      <AuthedStack.Screen name="Home" options={{ headerShown: false }}>
        {({ navigation }) => (
          <HomeScreen
            dubbridgeEnv={dubbridgeEnv}
            gatewayBaseUrl={gatewayBaseUrl}
            onOpenAssets={() => navigation.navigate("AssetList")}
            onOpenUpload={() => navigation.navigate("Upload")}
            onOpenReview={() => navigation.navigate("ReviewInbox")}
            onOpenOrganizations={() => navigation.navigate("OrganizationList")}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="AssetList" options={{ title: "Assets" }}>
        {({ navigation }) => (
          <AssetListScreen
            gatewayBaseUrl={gatewayBaseUrl}
            onOpenAsset={(asset) =>
              navigation.navigate("AssetDetail", {
                assetId: asset.id,
                assetTitle: asset.title,
              })
            }
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="AssetDetail"
        options={({ route }) => ({ title: route.params.assetTitle })}
      >
        {({ route, navigation }) => (
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
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="Compliance"
        options={({ route }) => ({ title: `${route.params.assetTitle} compliance` })}
      >
        {({ route, navigation }) => (
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
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="Consent"
        options={({ route }) => ({ title: `${route.params.assetTitle} consent` })}
      >
        {({ route }) => <ConsentScreen assetId={route.params.assetId} gatewayBaseUrl={gatewayBaseUrl} />}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="Upload" options={{ title: "Upload" }}>
        {({ navigation }) => (
          <UploadScreen
            gatewayBaseUrl={gatewayBaseUrl}
            onSuccess={() => navigation.navigate("AssetList")}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="OrganizationList" options={{ title: "Organizations" }}>
        {({ navigation }) => (
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
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="OrganizationMembers"
        options={({ route }) => ({ title: `${route.params.orgName} members` })}
      >
        {({ route }) => (
          <OrganizationMembersScreen
            gatewayBaseUrl={gatewayBaseUrl}
            orgId={route.params.orgId}
            viewerRole={route.params.viewerRole}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="ProjectList"
        options={({ route }) => ({ title: route.params.orgName })}
      >
        {({ route, navigation }) => (
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
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="ProjectDetail"
        options={({ route }) => ({ title: route.params.projectName })}
      >
        {({ route, navigation }) => (
          <ProjectDetailScreen
            gatewayBaseUrl={gatewayBaseUrl}
            orgId={route.params.orgId}
            projectId={route.params.projectId}
            onOpenAsset={(assetId, assetTitle) =>
              navigation.navigate("AssetDetail", { assetId, assetTitle })
            }
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="ReviewInbox" options={{ title: "Review inbox" }}>
        {({ navigation, route }) => (
          <ReviewInboxScreen
            gatewayBaseUrl={gatewayBaseUrl}
            initialTaskId={route.params?.initialTaskId ?? null}
            onOpenTask={(task) => navigation.navigate("ReviewDetail", { task })}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="ReviewDetail" options={{ title: "Review task" }}>
        {({ route, navigation }) => (
          <ReviewDetailScreen
            task={route.params.task}
            gatewayBaseUrl={gatewayBaseUrl}
            onBack={() => navigation.navigate("ReviewInbox")}
          />
        )}
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

  useEffect(() => {
    const subscription = Notifications.addNotificationResponseReceivedListener(
      (response) => {
        const data = response.notification.request.content.data as Record<string, unknown>;
        const refEntityType = data["ref_entity_type"];
        const refEntityId = data["ref_entity_id"];

        if (refEntityType !== "review_task" || typeof refEntityId !== "string") {
          return;
        }

        if (auth.status !== "authed") {
          // Logged out: store intent; the authed navigator will pick it up on mount.
          setPendingDeepLink({ taskId: refEntityId });
          return;
        }

        // Authed: navigate immediately if the navigator is ready.
        if (navRef.current?.isReady()) {
          navigateToReviewInbox(navRef.current, refEntityId);
        } else {
          setPendingDeepLink({ taskId: refEntityId });
        }
      },
    );
    return () => subscription.remove();
  }, [auth.status]);

  useEffect(() => {
    if (!runtimeConfig.ok || auth.status !== "authed") {
      return;
    }

    const client = createGatewayClient({
      gatewayBaseUrl: runtimeConfig.value.gatewayBaseUrl,
    });
    void registerPush(client, auth.sessionRef);
  }, [auth.sessionRef, auth.status, runtimeConfig]);

  function onNavReady() {
    if (pendingDeepLink && auth.status === "authed" && navRef.current) {
      navigateToReviewInbox(navRef.current, pendingDeepLink.taskId);
      setPendingDeepLink(null);
    }
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

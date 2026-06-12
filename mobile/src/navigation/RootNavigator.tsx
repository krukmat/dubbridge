import { NavigationContainer } from "@react-navigation/native";
import { createNativeStackNavigator } from "@react-navigation/native-stack";

import { AuthProvider, useAuth } from "../auth/AuthProvider";
import { readRuntimeConfig } from "../config/env";
import { AssetDetailScreen } from "../screens/AssetDetailScreen";
import { AssetListScreen } from "../screens/AssetListScreen";
import { ConfigErrorScreen } from "../screens/ConfigErrorScreen";
import { HomeScreen } from "../screens/HomeScreen";
import { LoginScreen } from "../screens/LoginScreen";
import { ProjectDetailScreen } from "../screens/ProjectDetailScreen";
import { ProjectListScreen } from "../screens/ProjectListScreen";
import { UploadScreen } from "../screens/UploadScreen";

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
  Upload: undefined;
  ProjectList: { orgId: string };
  ProjectDetail: { orgId: string; projectId: string; projectName: string };
};

const UnauthedStack = createNativeStackNavigator<UnauthedStackParamList>();
const AuthedStack = createNativeStackNavigator<AuthedStackParamList>();

function UnauthedNavigator() {
  return (
    <UnauthedStack.Navigator>
      <UnauthedStack.Screen name="Login" options={{ title: "DubBridge" }}>
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
    <AuthedStack.Navigator>
      <AuthedStack.Screen name="Home" options={{ title: "Home" }}>
        {({ navigation }) => (
          <HomeScreen
            dubbridgeEnv={dubbridgeEnv}
            gatewayBaseUrl={gatewayBaseUrl}
            onOpenAssets={() => navigation.navigate("AssetList")}
            onOpenUpload={() => navigation.navigate("Upload")}
            onOpenProjects={(orgId) => navigation.navigate("ProjectList", { orgId })}
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
        {({ route }) => (
          <AssetDetailScreen
            assetId={route.params.assetId}
            gatewayBaseUrl={gatewayBaseUrl}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen name="Upload" options={{ title: "Upload" }}>
        {({ navigation }) => (
          <UploadScreen
            gatewayBaseUrl={gatewayBaseUrl}
            onSuccess={() => navigation.navigate("AssetList")}
          />
        )}
      </AuthedStack.Screen>
      <AuthedStack.Screen
        name="ProjectList"
        options={{ title: "Projects" }}
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
    </AuthedStack.Navigator>
  );
}

function RootNavigatorContent() {
  const auth = useAuth();
  const runtimeConfig = readRuntimeConfig();

  if (!runtimeConfig.ok) {
    return <ConfigErrorScreen message={runtimeConfig.message} />;
  }

  return (
    <NavigationContainer>
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

export function RootNavigator() {
  return (
    <AuthProvider>
      <RootNavigatorContent />
    </AuthProvider>
  );
}

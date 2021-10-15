namespace System.Reflection
{
    public class AssemblyCompanyAttribute : Attribute
    {
        public string Company { get; set; }

        public AssemblyCompanyAttribute(string company)
        {
            this.Company = company;
        }
    }

    public class AssemblyConfigurationAttribute : Attribute
    {
        public string Configuration { get; set; }

        public AssemblyConfigurationAttribute(string configuration)
        {
            this.Configuration = configuration;
        }
    }

    public class AssemblyFileVersionAttribute : Attribute
    {
        public string Version { get; set; }

        public AssemblyFileVersionAttribute(string version)
        {
            this.Version = version;
        }
    }

    public class AssemblyInformationalVersionAttribute : Attribute
    {
        public string Infomation { get; set; }

        public AssemblyInformationalVersionAttribute(string infomation)
        {
            this.Infomation = infomation;
        }
    }

    public class AssemblyProductAttribute : Attribute
    {
        public string Product { get; set; }

        public AssemblyProductAttribute(string product)
        {
            this.Product = product;
        }
    }

    public class AssemblyTitleAttribute : Attribute
    {
        public string Title { get; set; }

        public AssemblyTitleAttribute(string title)
        {
            this.Title = title;
        }
    }

    public class AssemblyVersionAttribute : Attribute
    {
        public string Version { get; set; }

        public AssemblyVersionAttribute(string version)
        {
            this.Version = version;
        }
    }
}
